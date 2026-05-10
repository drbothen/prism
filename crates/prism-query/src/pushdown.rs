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
