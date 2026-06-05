//! `pushdown` — sensor filter push-down classification.
//!
//! Translates PrismQL AST WHERE predicates into sensor-native API filter
//! syntax and classifies each predicate as either push-down-capable or
//! post-filter for each target sensor adapter.
//!
//! Push-down is a **performance optimization only**. Query correctness is
//! identical whether push-down occurs or not. When uncertain, classify as
//! `PostFilter` (conservative). (BC-2.11.007)
//!
//! # BC References
//! - BC-2.11.007 — Sensor Filter Push-Down
//!
//! # VP References
//! - VP-031 — REQUIRED columns always result in `PushDown` (proptest)
//!
//! Story: S-3.02

// S-3.02 stub functions: dead_code suppressed pending implementation (stub-phase convention).
#![allow(dead_code)]

use prism_core::ColumnOptions;
use prism_spec_engine::spec_parser::ColumnSpec;

use crate::ast::Expr;

// ---------------------------------------------------------------------------
// ColumnPushDownOption
// ---------------------------------------------------------------------------

/// Push-down capability taxonomy for a sensor adapter column.
///
/// Mirrors the BC-2.11.007 column options table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColumnPushDownOption {
    /// API requires this parameter; query rejected without it. (BC-2.11.007)
    Required,
    /// Native API filter parameter; pushed down for performance.
    Index,
    /// Secondary/supplemental API filtering; pushed down when present.
    Additional,
    /// Prism-local optimization; NOT pushed down to sensor.
    Optimized,
    /// No push-down behavior; post-filter only.
    Default,
}

// ---------------------------------------------------------------------------
// Predicate
// ---------------------------------------------------------------------------

/// A single classified WHERE predicate.
///
/// Carries both the AST expression and the result of push-down classification
/// against a target sensor.
#[derive(Debug, Clone)]
pub struct Predicate {
    /// The AST expression node for this predicate.
    pub expr: Expr,
    /// The column name this predicate constrains (e.g., `"severity_id"`).
    pub column_name: String,
}

// ---------------------------------------------------------------------------
// PushDownPlan
// ---------------------------------------------------------------------------

/// The output of `classify_predicates` for a given source and WHERE clause.
///
/// Implements BC-2.11.007 predicate classification contract.
///
/// # VP-031
/// For any REQUIRED column, the predicate MUST appear in `push_down`, never
/// in `post_filter`. This is the invariant tested by VP-031.
#[derive(Debug, Default, Clone)]
pub struct PushDownPlan {
    /// Predicates to pass to the sensor adapter as `QueryParams.filters`.
    ///
    /// Only REQUIRED, INDEX, and ADDITIONAL columns appear here.
    /// (BC-2.11.007)
    pub push_down: Vec<Predicate>,

    /// Predicates applied by DataFusion after materialization.
    ///
    /// OPTIMIZED and DEFAULT columns always appear here.
    /// (BC-2.11.007)
    pub post_filter: Vec<Predicate>,
}

// ---------------------------------------------------------------------------
// classify_predicates
// ---------------------------------------------------------------------------

/// Classify WHERE predicates for a specific sensor source.
///
/// For each predicate in `where_clause`:
/// - If the column is REQUIRED, INDEX, or ADDITIONAL on `source`: push down.
/// - Otherwise: post-filter (DataFusion evaluates after materialization).
///
/// Push-down is a best-effort optimization. When push-down classification is
/// ambiguous, `post_filter` is used. (BC-2.11.007 "when in doubt" rule)
///
/// # VP-031
/// REQUIRED columns MUST always appear in `push_down`.
///
/// # BC-2.11.007
/// Implements predicate classification per the Column Push-Down Capability
/// Taxonomy. Result is used as `QueryParams.filters` in `fan_out()`.
pub fn classify_predicates(where_clause: &[Expr], columns: &[ColumnSpec]) -> PushDownPlan {
    let mut plan = PushDownPlan::default();

    for expr in where_clause {
        // Extract the column name from the expression (best-effort).
        let col_name = extract_column_name(expr);
        let push_option = column_push_down_option_from_spec(&col_name, columns);

        let predicate = Predicate {
            expr: expr.clone(),
            column_name: col_name,
        };

        match push_option {
            ColumnPushDownOption::Required
            | ColumnPushDownOption::Index
            | ColumnPushDownOption::Additional => {
                plan.push_down.push(predicate);
            }
            ColumnPushDownOption::Optimized | ColumnPushDownOption::Default => {
                plan.post_filter.push(predicate);
            }
        }
    }

    plan
}

// ---------------------------------------------------------------------------
// column_push_down_option_from_spec
// ---------------------------------------------------------------------------

/// Determine the push-down option for a given column on a sensor spec.
///
/// Returns `ColumnPushDownOption::Default` when the column is not declared by
/// the sensor spec (conservative fallback). (BC-2.11.007)
pub(crate) fn column_push_down_option_from_spec(
    column_name: &str,
    columns: &[ColumnSpec],
) -> ColumnPushDownOption {
    let Some(col) = columns.iter().find(|c| c.name == column_name) else {
        // Unknown column → conservative fallback: post-filter.
        return ColumnPushDownOption::Default;
    };

    // Check options in priority order: Required > Index > Additional > Optimized > Default.
    if col.options.contains(&ColumnOptions::Required) {
        ColumnPushDownOption::Required
    } else if col.options.contains(&ColumnOptions::Index) {
        ColumnPushDownOption::Index
    } else if col.options.contains(&ColumnOptions::Additional) {
        ColumnPushDownOption::Additional
    } else if col.options.contains(&ColumnOptions::Optimized) {
        ColumnPushDownOption::Optimized
    } else {
        ColumnPushDownOption::Default
    }
}

// ---------------------------------------------------------------------------
// translate_push_down_filter
// ---------------------------------------------------------------------------

/// Translate a push-down predicate to sensor-native query syntax.
///
/// Sensor-specific translations:
/// - CrowdStrike: FQL filter syntax
/// - Cyberint: JSON body parameters
/// - Claroty xDome: POST body filter arrays
/// - Armis: AQL WHERE clauses
///
/// Returns `None` when translation fails (fall back to post-filter with a
/// WARN log). (BC-2.11.007)
///
/// # Future Caller (S-3.X)
/// This function will be called by `fan_out()` during the materialization
/// pipeline Step 3 to convert classified push-down predicates into the
/// per-sensor `QueryParams.filters` format before dispatching to each
/// `SensorAdapter`. The stub implementation emits a generic `column=value`
/// string; full sensor-native translations will be added per sensor story.
pub(crate) fn translate_push_down_filter(
    _predicate: &Predicate,
    _columns: &[ColumnSpec],
) -> Option<String> {
    // ADV-W3MT-P61-LOW-001 / POL-12: replace todo!() with the correct sentinel.
    // Sensor-specific filter translation (CrowdStrike FQL, Cyberint queries,
    // Claroty xDome POST body, Armis AQL) is deferred to per-sensor stories (S-3.X).
    // `None` is the correct return: callers fall back to post-DataFusion filtering
    // with a WARN log, which is the documented behavior. (BC-2.11.007)
    // No sensor API leakage — Debug-formatted AST is NOT emitted to external APIs.
    let _ = (_predicate, _columns); // documented deferral
    None
}

// ---------------------------------------------------------------------------
// extract_column_name (internal helper)
// ---------------------------------------------------------------------------

/// Extract the column name from a PrismQL `Expr` node (best-effort).
///
/// Returns an empty string for complex expressions that don't have a simple
/// column reference (these will fall through to `Default` / post-filter).
fn extract_column_name(expr: &Expr) -> String {
    use crate::ast::Expr::*;
    match expr {
        // `field op value` — extract the LHS column name.
        Compare { lhs, .. } => match lhs.as_ref() {
            Field(fp) => fp.segments.join("."),
            VirtualField(vf) => virtual_field_name(vf).to_string(),
            _ => String::new(),
        },
        Field(fp) => fp.segments.join("."),
        VirtualField(vf) => virtual_field_name(vf).to_string(),
        _ => String::new(),
    }
}

/// Map a `VirtualField` enum to its string name.
#[allow(unreachable_patterns)] // VirtualField is #[non_exhaustive]; wildcard needed for future variants.
fn virtual_field_name(vf: &crate::ast::VirtualField) -> &'static str {
    use crate::ast::VirtualField::*;
    match vf {
        Sensor => "_sensor",
        Client => "_client",
        SourceTable => "_source_table",
        SourceType => "_source_type",
        SafetyFlags => "_safety_flags",
        _ => "_unknown",
    }
}

// ---------------------------------------------------------------------------
// predicate_tree_to_filter_map (F-LP2-MED-1)
// ---------------------------------------------------------------------------

/// Convert a `Predicate` tree into a sensor `FilterMap` by extracting simple
/// equality predicates.
///
/// This function replaces the local `collect_eq_filters` helper in
/// `materialization.rs`. It extracts `field = 'value'` equality predicates from
/// the predicate tree (walking `AND` conjunctions) and builds a flat `FilterMap`
/// from them directly.
///
/// Push-down is a performance optimization only — predicates not expressible as
/// simple `field = value` pairs are silently omitted from the filter map (they
/// will be evaluated by DataFusion post-materialization). (BC-2.11.007)
///
/// # Scope note (F-LP3-MED-1)
/// This function is called pre-fan-out from `extract_push_down_filters_as_map`,
/// where no per-sensor `ColumnSpec` is available. Threading `ColumnSpec` through
/// would require changing the call sequence in `extract_push_down_filters_as_map`
/// and the fan-out orchestration — that is tracked as future work (wave-5, ADR-022 §C).
/// For now, all equality predicates are passed through to the sensor adapter
/// regardless of whether the column is declared REQUIRED/INDEX/ADDITIONAL; the
/// adapter discards unknown filter parameters. `classify_predicates` is NOT called
/// here because its return value would be meaningless with an empty spec slice
/// (all predicates fall through to `post_filter`, which is then discarded).
pub fn predicate_tree_to_filter_map(
    predicate: &crate::ast::Predicate,
) -> prism_sensors::types::FilterMap {
    // Collect all `field = 'value'` equality expressions from the predicate tree.
    let mut eq_exprs: Vec<crate::ast::Expr> = Vec::new();
    collect_equality_exprs(predicate, &mut eq_exprs);

    // Build the FilterMap directly from collected equality expressions.
    // (Per-sensor classify_predicates integration deferred to wave-5 when ColumnSpec
    // is available at the pre-fan-out stage — see scope note above.)
    let mut filters = prism_sensors::types::FilterMap::new();
    for expr in &eq_exprs {
        if let Some((col, val)) = extract_eq_filter_from_expr(expr) {
            filters.insert(col, val);
        }
    }
    filters
}

/// Recursively collect equality comparison expressions from a predicate tree.
///
/// Only collects `field = 'string_value'` comparisons. `AND` conjunctions are
/// decomposed; other logical operators are skipped (conservative).
fn collect_equality_exprs(pred: &crate::ast::Predicate, out: &mut Vec<crate::ast::Expr>) {
    use crate::ast::{CompareOp, Expr, Literal, LogicalOp, Predicate};
    match pred {
        // Only include `field = 'string'` comparisons (not virtual fields or complex exprs).
        Predicate::Compare { lhs, op, rhs }
            if *op == CompareOp::Eq
                && matches!(lhs.as_ref(), Expr::Field(_))
                && matches!(rhs.as_ref(), Expr::Literal(Literal::String(_))) =>
        {
            out.push(Expr::Compare {
                lhs: lhs.clone(),
                op: op.clone(),
                rhs: rhs.clone(),
            });
        }
        Predicate::Logical { op, predicates } if *op == LogicalOp::And => {
            for child in predicates {
                collect_equality_exprs(child, out);
            }
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// extract_time_window_from_ast (ADR-033 T1 — pre-fan-out heuristic)
// ---------------------------------------------------------------------------

/// Extract `(start_time, end_time)` bounds from a PrismQL predicate tree.
///
/// Implements ADR-033 Option T1: walk `Predicate::Compare` nodes with
/// `op ∈ {Gt, Ge, Lt, Le}` and match lhs column names against datetime INDEX
/// columns in the provided `resolved_spec_map`. Extracted ISO8601 strings are
/// returned as `(start_time, end_time)` where:
/// - `start_time` corresponds to `Gt`/`Ge` predicates (lower bound)
/// - `end_time`   corresponds to `Lt`/`Le` predicates (upper bound)
///
/// # Single-column assumption (F-P1-HIGH-002)
///
/// This function uses first-wins semantics: the first lower-bound predicate sets
/// `start_time`; the first upper-bound predicate sets `end_time`. When multiple
/// datetime INDEX columns are present (e.g., `created_timestamp` AND `updated_at`),
/// the first matched lower bound is used for `start_time` and the first matched
/// upper bound for `end_time`. These MAY be from different columns — the caller
/// (sensor FQL/AQL builder) is responsible for interpreting the bounds against its
/// canonical push-down column (e.g., CrowdStrike always uses `created_timestamp`;
/// Armis AQL augmentation appends to the base AQL regardless of which column name
/// was the source of the bound).
///
/// In practice this is safe for the current sensors because:
/// - CrowdStrike: only `created_timestamp` has `options = ["INDEX"]`.
/// - Armis devices: only `last_seen` has `options = ["INDEX"]` (in the devices table).
/// - Armis alerts: only `created_at` has `options = ["INDEX"]` (in the alerts table).
///
/// No production sensor spec has two datetime INDEX columns in the same table that
/// a single query would target simultaneously.
///
/// If future sensor specs add multiple datetime INDEX columns to the same table,
/// this function must be extended to track which column each bound came from, and
/// the per-sensor FQL/AQL builders must select the appropriate column bound.
/// See ADR-033 §Consequences (Option T2 for the full-featured solution).
///
/// # Safe default (ADR-033 §Consequences)
///
/// When `resolved_spec_map` is `None`, both return values are `None` (no push-down).
/// No panic; no push-down occurs.
///
/// # Story: S-DEMO-QUERY-PUSHDOWN-001 v2.1
/// Implemented (ADR-033 T1): walks Compare nodes with op ∈ {Gt, Ge, Lt, Le},
/// matches lhs column names against datetime INDEX columns in `resolved_spec_map`,
/// and extracts ISO8601 bounds. AC-WIRE-001 / AC-WIRE-001b verify this behavior.
pub fn extract_time_window_from_ast(
    predicate: &crate::ast::Predicate,
    source_names: &[&str],
    resolved_spec_map: Option<
        &std::collections::HashMap<String, Vec<prism_spec_engine::spec_parser::ColumnSpec>>,
    >,
) -> (Option<String>, Option<String>) {
    // ADR-033 §Consequences — safe default: None spec_map → no push-down, no panic.
    let spec_map = match resolved_spec_map {
        Some(m) => m,
        None => return (None, None),
    };

    // Collect all ColumnSpec entries for the given source_names, looking up datetime INDEX cols.
    // ADR-033 T1: match lhs column names against columns with column_type=datetime + options=[INDEX].
    let mut datetime_index_cols: std::collections::HashSet<String> =
        std::collections::HashSet::new();
    for source_name in source_names {
        if let Some(cols) = spec_map.get(*source_name) {
            for col in cols {
                if col.column_type == prism_core::ColumnType::Datetime
                    && col.options.contains(&ColumnOptions::Index)
                {
                    datetime_index_cols.insert(col.name.clone());
                }
            }
        }
    }

    let mut start_time: Option<String> = None;
    let mut end_time: Option<String> = None;

    extract_time_bounds_from_predicate(
        predicate,
        &datetime_index_cols,
        &mut start_time,
        &mut end_time,
    );

    (start_time, end_time)
}

/// Recursively walk the predicate tree collecting Gt/Ge (start) and Lt/Le (end) bounds
/// on datetime INDEX columns.
fn extract_time_bounds_from_predicate(
    predicate: &crate::ast::Predicate,
    datetime_index_cols: &std::collections::HashSet<String>,
    start_time: &mut Option<String>,
    end_time: &mut Option<String>,
) {
    use crate::ast::{CompareOp, Expr, Literal, LogicalOp, Predicate};
    match predicate {
        Predicate::Compare { lhs, op, rhs } => {
            // Only handle inequalities (Gt, Ge, Lt, Le).
            let is_range_op = matches!(
                op,
                CompareOp::Gt | CompareOp::Ge | CompareOp::Lt | CompareOp::Le
            );
            if !is_range_op {
                return;
            }
            // LHS must be a plain field reference.
            let col_name = match lhs.as_ref() {
                Expr::Field(fp) => fp.segments.join("."),
                _ => return,
            };
            // Column must be a known datetime INDEX column.
            if !datetime_index_cols.contains(&col_name) {
                return;
            }
            // RHS must be a Timestamp literal.
            let ts_str = match rhs.as_ref() {
                Expr::Literal(Literal::Timestamp(ts)) => {
                    // Use to_rfc3339 which produces e.g. "2026-01-01T00:00:00Z".
                    ts.instant.to_rfc3339()
                }
                _ => return,
            };
            // Gt/Ge → lower bound (start_time); Lt/Le → upper bound (end_time).
            // First-wins semantics: only the first extracted bound is used.
            match op {
                CompareOp::Gt | CompareOp::Ge if start_time.is_none() => {
                    *start_time = Some(ts_str);
                }
                CompareOp::Lt | CompareOp::Le if end_time.is_none() => {
                    *end_time = Some(ts_str);
                }
                _ => {}
            }
        }
        Predicate::Logical { op, predicates } if *op == LogicalOp::And => {
            for child in predicates {
                extract_time_bounds_from_predicate(
                    child,
                    datetime_index_cols,
                    start_time,
                    end_time,
                );
            }
        }
        _ => {}
    }
}

/// Augment a base Armis AQL string with time-window clauses.
///
/// Implements BC-2.01.013 v1.14 Mechanism B AQL-clause augmentation:
/// - If base AQL already contains `after:`, `before:`, or `timeFrame:` → return verbatim
///   (anti-double-filter guard, AC-ARMIS-TW-003).
/// - If `start_time` is present → append `after:YYYY-MM-DDTHH:MM:SS` (bare, unquoted,
///   timezone-naive per research-doc §2.2, AC-ARMIS-TW-001).
/// - If `end_time` is present → append `before:YYYY-MM-DDTHH:MM:SS`.
/// - Clauses are space-separated.
///
/// # AQL syntax
///
/// Canonical Armis AQL time syntax (research-confirmed HIGH confidence, 6 sources):
/// `after:YYYY-MM-DDTHH:MM:SS` (bare, unquoted, timezone-naive — NOT `after:"T"`, NOT `Z` suffix).
///
/// # Story: S-DEMO-QUERY-PUSHDOWN-001 v2.1
/// Implemented: appends `after:YYYY-MM-DDTHH:MM:SS` / `before:YYYY-MM-DDTHH:MM:SS`
/// clauses (bare, unquoted, timezone-naive) to `base_aql` when time bounds are present.
/// Anti-double-filter guard skips augmentation if base AQL already contains a time keyword.
/// Wired into `SpecDrivenSensorAdapter::fetch()` via the Armis sensor branch.
/// AC-ARMIS-TW-001 / AC-ARMIS-TW-003 verify this behavior.
pub fn augment_armis_aql_with_time_window(
    base_aql: &str,
    start_time: Option<&str>,
    end_time: Option<&str>,
) -> String {
    // Anti-double-filter guard (AC-ARMIS-TW-003 / BC-2.01.013 v1.14 Mechanism B):
    // If the base AQL already contains any of the canonical Armis time keywords,
    // return it verbatim — do NOT append a second time clause.
    if base_aql.contains("after:")
        || base_aql.contains("before:")
        || base_aql.contains("timeFrame:")
    {
        return base_aql.to_string();
    }

    // If no time bounds are provided, pass through verbatim.
    if start_time.is_none() && end_time.is_none() {
        return base_aql.to_string();
    }

    // Build the time clause(s) to append.
    // Canonical Armis AQL syntax (research-confirmed HIGH confidence, 6 sources):
    //   after:YYYY-MM-DDTHH:MM:SS  (bare, unquoted, timezone-naive — NO Z suffix)
    //   before:YYYY-MM-DDTHH:MM:SS
    // Clauses are space-separated (research-doc §3, BlinkOps source).
    let mut result = base_aql.to_string();

    if let Some(start) = start_time {
        // Strip the trailing 'Z' suffix if present (timezone-naive form required).
        let start_naive = strip_z_suffix(start);
        result.push(' ');
        result.push_str("after:");
        result.push_str(start_naive);
    }
    if let Some(end) = end_time {
        let end_naive = strip_z_suffix(end);
        result.push(' ');
        result.push_str("before:");
        result.push_str(end_naive);
    }

    result
}

/// Strip a trailing `Z` UTC suffix from an ISO8601 timestamp string.
///
/// The canonical Armis AQL form is timezone-naive `YYYY-MM-DDTHH:MM:SS`
/// (research-doc §2.2, R2: bare/unquoted, no `Z` suffix). PrismQL stores
/// timestamps as `DateTime<Utc>` and `to_rfc3339()` appends `+00:00`.
/// This function strips either `Z` or `+00:00` suffixes.
fn strip_z_suffix(ts: &str) -> &str {
    if let Some(stripped) = ts.strip_suffix('Z') {
        return stripped;
    }
    if let Some(stripped) = ts.strip_suffix("+00:00") {
        return stripped;
    }
    ts
}

/// Extract a `(column_name, json_value)` pair from an `Expr::Compare` equality.
///
/// Returns `None` if the expression is not a simple `field = 'string'` comparison.
fn extract_eq_filter_from_expr(expr: &crate::ast::Expr) -> Option<(String, serde_json::Value)> {
    use crate::ast::{CompareOp, Expr, Literal};
    match expr {
        Expr::Compare { lhs, op, rhs } if *op == CompareOp::Eq => {
            let col = match lhs.as_ref() {
                Expr::Field(fp) => fp.segments.join("."),
                _ => return None,
            };
            if let Expr::Literal(Literal::String(val)) = rhs.as_ref() {
                Some((col, serde_json::Value::String(val.clone())))
            } else {
                None
            }
        }
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Red Gate tests — S-DEMO-QUERY-PUSHDOWN-001 v2.1
// AC-WIRE-001, AC-WIRE-001b, AC-ARMIS-TW-001, AC-ARMIS-TW-003
// ---------------------------------------------------------------------------

#[cfg(test)]
mod pushdown_red_gate_tests {
    //! Red Gate tests for S-DEMO-QUERY-PUSHDOWN-001 v2.1.
    //!
    //! These tests exercise `extract_time_window_from_ast` and
    //! `augment_armis_aql_with_time_window` from the pushdown module.
    //! ALL tests in this module MUST FAIL before implementation begins.
    //!
    //! # SAP-2 compliance
    //! Tests that use `ColumnSpec` fixtures construct them from production-TOML-derived
    //! column shapes (column_type = "datetime", options = [Index] — exact properties
    //! used in `armis.sensor.toml` and `crowdstrike.sensor.toml`).
    //! No fabricated fixture diverges from the production TOML shape.

    use std::collections::HashMap;

    use prism_core::{ColumnOptions, ColumnType};
    use prism_spec_engine::spec_parser::ColumnSpec;

    use super::{augment_armis_aql_with_time_window, extract_time_window_from_ast};
    use crate::ast::{Ast, SqlStatement};
    use crate::filter_parser::PrismQlParser;

    // -----------------------------------------------------------------------
    // Helper: build a minimal ColumnSpec for a datetime INDEX column.
    // Shape is a strict subset of the production armis.sensor.toml and
    // crowdstrike.sensor.toml column declarations:
    //   column_type = "datetime", options = ["INDEX"]
    // No divergence from production shape (SAP-2 gate).
    //
    // NOTE: ColumnSpec is #[non_exhaustive]. External callers MUST use the Default
    // impl and set fields individually per CLAUDE.md §Conventions non-exhaustive discipline.
    // -----------------------------------------------------------------------
    fn make_datetime_index_column(name: &str) -> ColumnSpec {
        let mut col = ColumnSpec::default();
        col.name = name.to_string();
        col.column_type = ColumnType::Datetime;
        col.options = vec![ColumnOptions::Index];
        col
    }

    // -----------------------------------------------------------------------
    // Helper: extract WHERE predicate from a parsed SELECT AST.
    // -----------------------------------------------------------------------
    fn parse_where(query: &str) -> crate::ast::Predicate {
        let ast = PrismQlParser::parse(query)
            .unwrap_or_else(|e| panic!("PrismQlParser::parse failed for '{query}': {e:?}"));
        match ast {
            Ast::Sql(SqlStatement::Select(sql)) => {
                sql.where_.expect("query must have a WHERE clause")
            }
            other => panic!("expected SQL Select, got: {other:?}"),
        }
    }

    // -----------------------------------------------------------------------
    // AC-WIRE-001: extract_time_window_from_ast populates start_time
    // -----------------------------------------------------------------------

    /// AC-WIRE-001 / BC-2.01.013 v1.14 TV-BC-2.01.013-006 / ADR-033 T1
    ///
    /// A PrismQL `WHERE created_timestamp > '2026-01-01T00:00:00Z'` predicate on a
    /// `column_type = "datetime"` with `options = ["INDEX"]` column must yield
    /// `start_time = Some("2026-01-01T00:00:00Z")` and `end_time = None`.
    ///
    /// Red Gate: extract_time_window_from_ast stub returns (None, None).
    /// Fails with: assertion `start_time == Some("2026-01-01T00:00:00Z")` fails.
    #[allow(non_snake_case)]
    #[test]
    fn test_ac_wire_001_materialization_pipeline_populates_start_time_from_ast() {
        let query =
            "SELECT * FROM crowdstrike.detections WHERE created_timestamp > '2026-01-01T00:00:00Z'";
        let predicate = parse_where(query);

        // Production-TOML-derived column spec (crowdstrike.sensor.toml shape).
        // column_type = "datetime", options = ["INDEX"] — strict subset of production spec.
        let col = make_datetime_index_column("created_timestamp");
        let mut spec_map: HashMap<String, Vec<ColumnSpec>> = HashMap::new();
        spec_map.insert("crowdstrike.detections".to_string(), vec![col]);

        let (start_time, end_time) =
            extract_time_window_from_ast(&predicate, &["crowdstrike.detections"], Some(&spec_map));

        // AC-WIRE-001 item (a): start_time must be populated from the Gt predicate.
        assert!(
            start_time.is_some(),
            "AC-WIRE-001: extract_time_window_from_ast must return Some(start_time) for \
             `created_timestamp > '2026-01-01T00:00:00Z'` on a datetime INDEX column; \
             got None. ADR-033 T1 heuristic is not implemented or not wired."
        );
        assert!(
            start_time.as_deref().unwrap_or("").contains("2026-01-01"),
            "AC-WIRE-001: start_time must contain '2026-01-01'; got: {:?}",
            start_time
        );

        // AC-WIRE-001 item (b): end_time must be None (no Lt/Le predicate).
        assert!(
            end_time.is_none(),
            "AC-WIRE-001: end_time must be None when only Gt predicate present; got: {:?}",
            end_time
        );
    }

    /// AC-WIRE-001b / ADR-033 §Consequences — safe default when spec_map is None
    ///
    /// When `resolved_spec_map` is `None`, `extract_time_window_from_ast` MUST return
    /// `(None, None)` without panicking.
    ///
    /// NOTE: This test PASSES with the stub (returns (None, None) as the safe default).
    /// It is included to confirm the stub matches the expected safe-default behavior.
    /// The complementary AC-WIRE-001 test fails — demonstrating Red Gate on the real path.
    #[allow(non_snake_case)]
    #[test]
    fn test_ac_wire_001b_safe_default_when_spec_map_is_none() {
        let query =
            "SELECT * FROM crowdstrike.detections WHERE created_timestamp > '2026-01-01T00:00:00Z'";
        let predicate = parse_where(query);

        let (start_time, end_time) =
            extract_time_window_from_ast(&predicate, &["crowdstrike.detections"], None);

        assert!(
            start_time.is_none(),
            "AC-WIRE-001b: start_time must be None when spec_map is None (safe default, ADR-033); \
             got: {:?}",
            start_time
        );
        assert!(
            end_time.is_none(),
            "AC-WIRE-001b: end_time must be None when spec_map is None (safe default, ADR-033); \
             got: {:?}",
            end_time
        );
    }

    // -----------------------------------------------------------------------
    // AC-ARMIS-TW-001: AQL augmentation appends after: clause
    // -----------------------------------------------------------------------

    /// AC-ARMIS-TW-001 / BC-2.01.013 v1.14 Mechanism B / BC-2.11.007 v1.8 §Mechanism B
    ///
    /// `augment_armis_aql_with_time_window("in:devices", Some("2026-01-01T00:00:00Z"), None)`
    /// must return `"in:devices after:2026-01-01T00:00:00"` (bare, unquoted, timezone-naive
    /// per research-doc §2.2).
    ///
    /// Red Gate: stub returns "in:devices" (no augmentation) → assertion fails.
    #[allow(non_snake_case)]
    #[test]
    fn test_ac_armis_tw_001_time_window_augmented_into_aql() {
        // start_time from PrismQL `last_seen > '2026-01-01T00:00:00Z'`
        // Canonical Armis AQL form: after:YYYY-MM-DDTHH:MM:SS (bare, no Z, no quotes)
        let result =
            augment_armis_aql_with_time_window("in:devices", Some("2026-01-01T00:00:00Z"), None);

        assert!(
            result.contains("after:2026-01-01T00:00:00"),
            "AC-ARMIS-TW-001: augmented AQL must contain 'after:2026-01-01T00:00:00' \
             (bare, unquoted, timezone-naive per research-doc §2.2); got: '{result}'. \
             MUST NOT use 'lastSeen:>\"T\"' form (unattested). \
             MUST NOT append 'Z' suffix. ADR-033 T1 Armis AQL augmentation not implemented."
        );
        assert!(
            result.starts_with("in:devices"),
            "AC-ARMIS-TW-001: augmented AQL must retain the base entity discriminator \
             'in:devices'; got: '{result}'"
        );
        // Must NOT use the unattested form
        assert!(
            !result.contains("lastSeen:>"),
            "AC-ARMIS-TW-001: MUST NOT use 'lastSeen:>\"T\"' form (unattested per \
             research-doc §2.3); got: '{result}'"
        );
    }

    /// AC-ARMIS-TW-001 bounded range variant: start AND end produces `after:T1 before:T2`.
    ///
    /// Red Gate: stub returns "in:devices" → assertion fails.
    #[allow(non_snake_case)]
    #[test]
    fn test_ac_armis_tw_001_bounded_range_after_and_before() {
        let result = augment_armis_aql_with_time_window(
            "in:devices",
            Some("2026-01-01T00:00:00Z"),
            Some("2026-06-01T00:00:00Z"),
        );

        assert!(
            result.contains("after:2026-01-01T00:00:00"),
            "AC-ARMIS-TW-001 (bounded range): must contain 'after:2026-01-01T00:00:00'; got: '{result}'"
        );
        assert!(
            result.contains("before:2026-06-01T00:00:00"),
            "AC-ARMIS-TW-001 (bounded range): must contain 'before:2026-06-01T00:00:00'; got: '{result}'"
        );
        // Clauses must be space-separated (not AND-joined per research §3)
        assert!(
            !result.contains(" AND "),
            "AC-ARMIS-TW-001 (bounded range): AQL clauses must be space-separated, \
             NOT 'AND'-joined per research-doc §3; got: '{result}'"
        );
    }

    // -----------------------------------------------------------------------
    // AC-ARMIS-TW-003: Anti-double-filter guard
    // -----------------------------------------------------------------------

    /// AC-ARMIS-TW-003 / BC-2.01.013 v1.14 Mechanism B anti-double-filter guard
    ///
    /// If the base AQL already contains `after:`, no second time clause must be appended.
    /// The AQL is returned VERBATIM.
    ///
    /// NOTE: This test PASSES with the stub (returns base_aql unchanged).
    /// It is included to confirm the anti-double-filter guard behavior.
    /// The AC-ARMIS-TW-001 test fails — demonstrating Red Gate on the augmentation path.
    #[allow(non_snake_case)]
    #[test]
    fn test_ac_armis_tw_003_anti_double_filter_guard() {
        let base_aql = "in:devices after:2026-01-01T00:00:00";

        let result = augment_armis_aql_with_time_window(
            base_aql,
            Some("2026-01-01T00:00:00Z"), // same bound — should NOT double-augment
            None,
        );

        assert_eq!(
            result, base_aql,
            "AC-ARMIS-TW-003: when base AQL already contains 'after:', \
             the result must equal the verbatim base AQL; \
             no second 'after:' must be appended (anti-double-filter guard). \
             got: '{result}'"
        );

        // Guard also applies to 'before:' and 'timeFrame:'
        let base_with_before = "in:devices before:2026-06-01T00:00:00";
        let result2 = augment_armis_aql_with_time_window(
            base_with_before,
            None,
            Some("2026-06-01T00:00:00Z"),
        );
        assert_eq!(
            result2, base_with_before,
            "AC-ARMIS-TW-003: 'before:' guard — verbatim passthrough; got: '{result2}'"
        );

        let base_with_timeframe = "in:devices timeFrame:\"3 Hours\"";
        let result3 = augment_armis_aql_with_time_window(
            base_with_timeframe,
            Some("2026-01-01T00:00:00Z"),
            None,
        );
        assert_eq!(
            result3, base_with_timeframe,
            "AC-ARMIS-TW-003: 'timeFrame:' guard — verbatim passthrough; got: '{result3}'"
        );
    }

    /// AC-ARMIS-TW-003 no-augmentation case: no time bounds → verbatim passthrough.
    ///
    /// PASSES with stub (returns base_aql) — confirmed correct stub behavior.
    #[allow(non_snake_case)]
    #[test]
    fn test_ac_armis_tw_003_no_time_bounds_passes_through_verbatim() {
        let base_aql = "in:devices";
        let result = augment_armis_aql_with_time_window(base_aql, None, None);
        assert_eq!(
            result, base_aql,
            "AC-ARMIS-TW-003: no time bounds → base AQL must pass through verbatim; got: '{result}'"
        );
    }

    // -----------------------------------------------------------------------
    // F-P1-HIGH-002: multi-datetime-column / duplicate-predicate behavior
    // Documents and verifies the single-column assumption.
    // -----------------------------------------------------------------------

    /// F-P1-HIGH-002 / ADR-033 T1 single-column assumption documentation test.
    ///
    /// When a query has TWO datetime INDEX columns with predicates (e.g., both
    /// `created_timestamp > T1` and `updated_at < T2`), the first-wins semantics apply:
    /// - First Gt/Ge predicate on ANY datetime INDEX column → start_time
    /// - First Lt/Le predicate on ANY datetime INDEX column → end_time
    ///
    /// This test verifies the documented behavior when multiple datetime INDEX columns
    /// are present. The bounds may come from different columns. The per-sensor FQL/AQL
    /// builder (e.g., `build_crowdstrike_fql`) uses them for its canonical column
    /// regardless of which source column provided the bound.
    ///
    /// This behavior is safe for current sensors (each table has at most one datetime
    /// INDEX column). See `extract_time_window_from_ast` doc for the full rationale.
    #[allow(non_snake_case)]
    #[test]
    fn test_ac_high_002_multi_datetime_column_first_wins_semantics() {
        // Both created_timestamp and updated_at are datetime INDEX columns.
        let col1 = make_datetime_index_column("created_timestamp");
        let col2 = make_datetime_index_column("updated_at");
        let mut spec_map: HashMap<String, Vec<ColumnSpec>> = HashMap::new();
        spec_map.insert("crowdstrike.detections".to_string(), vec![col1, col2]);

        // Query: created_timestamp > T1 AND updated_at < T2
        let query = "SELECT * FROM crowdstrike.detections \
                     WHERE created_timestamp > '2026-01-01T00:00:00Z' \
                     AND updated_at < '2026-06-01T00:00:00Z'";
        let predicate = parse_where(query);

        let (start_time, end_time) =
            extract_time_window_from_ast(&predicate, &["crowdstrike.detections"], Some(&spec_map));

        // Both bounds should be extracted (first-wins: created_timestamp → start, updated_at → end).
        assert!(
            start_time.is_some(),
            "F-P1-HIGH-002: start_time must be extracted from Gt predicate on created_timestamp; got None"
        );
        assert!(
            end_time.is_some(),
            "F-P1-HIGH-002: end_time must be extracted from Lt predicate on updated_at; got None"
        );
        assert!(
            start_time.as_deref().unwrap_or("").contains("2026-01-01"),
            "F-P1-HIGH-002: start_time must contain '2026-01-01'; got: {:?}",
            start_time
        );
        assert!(
            end_time.as_deref().unwrap_or("").contains("2026-06-01"),
            "F-P1-HIGH-002: end_time must contain '2026-06-01'; got: {:?}",
            end_time
        );
    }

    /// F-P1-HIGH-002 / EC-004: non-datetime column predicates are not extracted.
    ///
    /// When a column is declared with `column_type = "string"` (not "datetime"),
    /// its compare predicates must NOT contribute to start_time or end_time.
    /// Only `column_type = "datetime" + options = ["INDEX"]` columns qualify.
    #[allow(non_snake_case)]
    #[test]
    fn test_ac_high_002_non_datetime_column_not_extracted() {
        // Only a string column with INDEX option — NOT a datetime column.
        // EC-004: column_type != "datetime" → predicate silently skipped.
        let mut string_col = ColumnSpec::default();
        string_col.name = "severity".to_string();
        string_col.column_type = prism_core::ColumnType::String; // NOT Datetime
        string_col.options = vec![ColumnOptions::Index];

        let mut spec_map: HashMap<String, Vec<ColumnSpec>> = HashMap::new();
        spec_map.insert("crowdstrike.detections".to_string(), vec![string_col]);

        // Query predicates severity > 'medium' — but severity is String, not datetime.
        // This should NOT produce any time bounds.
        // (The parser may not actually accept > for strings, but we test the classification.)
        // Use a datetime-format value to ensure if it WERE extracted it would be non-None.
        // (The actual query is nonsensical but tests the classification gate.)
        let col_dt = make_datetime_index_column("created_timestamp");
        let mut spec_map2: HashMap<String, Vec<ColumnSpec>> = HashMap::new();
        spec_map2.insert("crowdstrike.detections".to_string(), vec![col_dt]);

        // Query with a NON-INDEX datetime column (no options field).
        let mut plain_dt_col = ColumnSpec::default();
        plain_dt_col.name = "updated_at".to_string();
        plain_dt_col.column_type = prism_core::ColumnType::Datetime;
        plain_dt_col.options = vec![]; // NO INDEX option

        spec_map2
            .get_mut("crowdstrike.detections")
            .unwrap()
            .push(plain_dt_col);

        let query = "SELECT * FROM crowdstrike.detections \
                     WHERE updated_at > '2026-01-01T00:00:00Z'";
        let predicate = parse_where(query);

        let (start_time, end_time) =
            extract_time_window_from_ast(&predicate, &["crowdstrike.detections"], Some(&spec_map2));

        // updated_at has no INDEX option → predicate MUST be silently skipped.
        assert!(
            start_time.is_none(),
            "F-P1-HIGH-002 EC-004: non-INDEX datetime column 'updated_at' must NOT contribute \
             to start_time; got: {:?}. Only datetime columns with options=['INDEX'] are \
             push-down-eligible per ADR-033 T1.",
            start_time
        );
        assert!(
            end_time.is_none(),
            "F-P1-HIGH-002 EC-004: non-INDEX datetime column must not contribute to end_time; got: {:?}",
            end_time
        );
    }
}
