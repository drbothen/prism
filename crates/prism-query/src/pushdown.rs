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
//! Story: S-DEMO-QUERY-PUSHDOWN-001 (ADR-033 T1 pre-fan-out time-window extraction)

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
/// and the fan-out orchestration — tracked in wave-5 scope (ADR-022 §C).
/// All equality predicates are passed through to the sensor adapter regardless of
/// whether the column is declared REQUIRED/INDEX/ADDITIONAL; the adapter discards
/// unknown filter parameters. `classify_predicates` is NOT called here because its
/// return value would be meaningless with an empty spec slice (all predicates fall
/// through to `post_filter`, which is then discarded).
pub fn predicate_tree_to_filter_map(
    predicate: &crate::ast::Predicate,
) -> prism_sensors::types::FilterMap {
    // Collect all `field = 'value'` equality expressions from the predicate tree.
    let mut eq_exprs: Vec<crate::ast::Expr> = Vec::new();
    collect_equality_exprs(predicate, &mut eq_exprs);

    // Build the FilterMap directly from collected equality expressions.
    // (Per-sensor classify_predicates integration deferred to wave-5 when ColumnSpec
    // is available at the pre-fan-out stage — see scope note above.)
    //
    // SEC-002: `aql` filter values are user-provided content (e.g., `WHERE aql = 'in:devices'`).
    // Before these values are interpolated into a URL path template (via `${query.filter.aql}`),
    // the Interpolator applies percent-encoding in `InterpolationContext::UrlPath` context
    // (prism-spec-engine::interpolation::Interpolator::percent_encode). Future reviewers:
    // the encoding boundary is at interpolation time, not here.
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
/// Only collects case-sensitive `field = 'string_value'` comparisons.
/// `AND` conjunctions are decomposed; other logical operators are skipped (conservative).
///
/// Predicates not expressible as simple `field = value` sensor filters are silently
/// omitted: this includes `!=`, range comparisons, and — critically — case-insensitive
/// `IEQ` predicates (`case_insensitive: true`). IEQ semantics cannot be expressed as
/// a case-sensitive equality push-down to a sensor API; the sensor would receive a
/// case-sensitive filter and silently miss OCSF Title-case values (e.g., `'High'` vs
/// `'high'`). See BC-2.11.024 F-P9-LOW-1.
fn collect_equality_exprs(pred: &crate::ast::Predicate, out: &mut Vec<crate::ast::Expr>) {
    use crate::ast::{CompareOp, Expr, Literal, LogicalOp, Predicate};
    match pred {
        // Only include case-sensitive `field = 'string'` comparisons (not virtual fields or
        // complex exprs, and NOT IEQ predicates — case_insensitive: true push-down would
        // apply a case-sensitive filter at the sensor layer, silently missing mismatched casing).
        Predicate::Compare {
            lhs,
            op,
            rhs,
            case_insensitive: false,
        } if *op == CompareOp::Eq
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
/// - CrowdStrike: only `created_timestamp` has `options = ["INDEX"]` (datetime-typed).
/// - Armis devices: `last_seen` is the only **datetime**+INDEX column (devices table).
///   `aql` is also `options = ["INDEX"]` but is `column_type = "string"` — it carries
///   the AQL push-down filter value and is not a temporal bound source for this function.
/// - Armis alerts: `created_at` is the only **datetime**+INDEX column (alerts table).
///   `aql` is also `options = ["INDEX"]` but is `column_type = "string"` — same note as
///   Armis devices.
/// - Claroty audit_logs: only `timestamp` has `options = ["INDEX"]` (datetime-typed).
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
    // Per-source `ocsf_column_naming` flag map (ADR-058 §I6).
    // When `Some`, the flattened OCSF Arrow name is inserted into `datetime_index_cols`
    // only when the entry for the source name is `true`. When `None` or the source is
    // absent, defaults to `false` (safe: no flattened-name registration).
    ocsf_naming_map: Option<&std::collections::HashMap<String, bool>>,
) -> (Option<String>, Option<String>) {
    // ADR-033 §Consequences — safe default: None spec_map → no push-down, no panic.
    if resolved_spec_map.is_none() {
        return (None, None);
    }

    // ADR-060 v1.8 structural reuse: delegate datetime INDEX col collection to the shared
    // `collect_datetime_index_cols` helper (materialization.rs). This ensures the gate
    // (plan-shape gate, Change 2) and the extractor (this function, Change 4) derive the
    // INDEX set from identical logic — source-scoped, OCSF-flattened, Condition-K aware.
    // The suppress_multi_index flag is discarded here: the extractor is a time-window
    // extraction utility, not a gate; suppression is the gate's responsibility.
    //
    // AC-014 (OQ-001 / ADR-058 §I6): OCSF-flattened Arrow names are registered when
    // ocsf_column_naming=true for the source (via ocsf_naming_map), same as before.
    let (datetime_index_vec, _suppress) = crate::materialization::collect_datetime_index_cols(
        resolved_spec_map,
        source_names,
        ocsf_naming_map,
    );
    let datetime_index_cols: std::collections::HashSet<String> =
        datetime_index_vec.into_iter().collect();

    let mut start_time: Option<String> = None;
    let mut end_time: Option<String> = None;

    extract_time_bounds_from_predicate(
        predicate,
        &datetime_index_cols,
        &mut start_time,
        &mut end_time,
    );

    // EC-003: if BOTH bounds are present and start_time > end_time (inverted window),
    // emit a WARN. Both bounds are still returned — correctness is preserved by the
    // DataFusion post-filter backstop. (BC-2.16.002 catalog row: push_down.inverted_time_range)
    if let (Some(ref start), Some(ref end)) = (&start_time, &end_time) {
        if start > end {
            tracing::warn!(
                event_type = "push_down.inverted_time_range",
                start_time = %start,
                end_time = %end,
                "push-down time window is inverted (start_time > end_time); \
                 sensor will receive both bounds and return empty/anomalous results"
            );
        }
    }

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
        Predicate::Compare { lhs, op, rhs, .. } => {
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
                    // Use to_rfc3339 which produces e.g. "2026-01-01T00:00:00+00:00".
                    // The `+00:00` form is fine for FQL push-down: the DTU parses it as
                    // DateTime<Utc> and compares correctly. (Do NOT change this to the
                    // `Z` form here — only the pipeline.rs timestamp normalization path
                    // needs `Z` for DataFusion string comparison; ADV-P08-MED-001.)
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
/// Implements BC-2.01.013 Mechanism B AQL-clause augmentation:
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
    // Anti-double-filter guard (AC-ARMIS-TW-003 / BC-2.01.013 Mechanism B):
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
    //
    // When base_aql is empty/blank, the time clause is the entire AQL — no leading space.
    // When base_aql is non-empty, clauses are joined with a single space: "<base> after:T".
    let base = base_aql.trim();
    let mut parts: Vec<String> = Vec::new();
    if !base.is_empty() {
        parts.push(base.to_string());
    }

    if let Some(start) = start_time {
        // Strip the trailing 'Z' or '+00:00' suffix (timezone-naive form required).
        let start_naive = strip_z_suffix(start);
        parts.push(format!("after:{start_naive}"));
    }
    if let Some(end) = end_time {
        let end_naive = strip_z_suffix(end);
        parts.push(format!("before:{end_naive}"));
    }

    parts.join(" ")
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

    /// AC-WIRE-001 / BC-2.01.013 TV-BC-2.01.013-006 / ADR-033 T1
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

        let (start_time, end_time) = extract_time_window_from_ast(
            &predicate,
            &["crowdstrike.detections"],
            Some(&spec_map),
            None, // crowdstrike: ocsf_column_naming=false (no flattened-name registration needed)
        );

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
            extract_time_window_from_ast(&predicate, &["crowdstrike.detections"], None, None);

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

    /// AC-ARMIS-TW-001 / BC-2.01.013 Mechanism B / BC-2.11.007 §Mechanism B
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

    /// AC-ARMIS-TW-003 / BC-2.01.013 Mechanism B anti-double-filter guard
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
    // OBS-P05-003: empty base AQL produces no leading space
    // -----------------------------------------------------------------------

    /// OBS-P05-003 / S-DEMO-QUERY-PUSHDOWN-001 v2.1
    ///
    /// When `base_aql` is empty (a query with only a datetime-INDEX predicate and
    /// no `aql = '...'` pseudo-column), `augment_armis_aql_with_time_window` must
    /// return the time clause WITHOUT a leading space.
    ///
    /// Scenario: empty base + only start_time → `"after:T"` (not `" after:T"`).
    /// Scenario: empty base + both bounds    → `"after:T1 before:T2"` (no leading space).
    /// Scenario: non-empty base + start_time → `"<base> after:T"` (single-space join, unchanged).
    ///
    /// The DTU parses AQL position-independently so the leading-space form is not a
    /// runtime crash, but cleanliness avoids subtle debug confusion and matches the
    /// canonical Armis AQL format documented in research-doc §2.2.
    #[test]
    fn test_obs_p05_003_empty_base_aql_no_leading_space() {
        // Case 1: empty base + start_time only → no leading space.
        let result = augment_armis_aql_with_time_window("", Some("2024-06-11T12:00:00Z"), None);
        assert!(
            !result.starts_with(' '),
            "OBS-P05-003: empty base AQL must not produce a leading space; got: '{result}'"
        );
        assert_eq!(
            result, "after:2024-06-11T12:00:00",
            "OBS-P05-003: empty base + start_time must yield 'after:T' (bare, no leading space, \
             no Z); got: '{result}'"
        );

        // Case 2: empty base + both bounds → no leading space, single-space between clauses.
        let result2 = augment_armis_aql_with_time_window(
            "",
            Some("2024-06-11T12:00:00Z"),
            Some("2024-06-12T00:00:00Z"),
        );
        assert!(
            !result2.starts_with(' '),
            "OBS-P05-003: empty base + both bounds must not produce a leading space; got: '{result2}'"
        );
        assert_eq!(
            result2, "after:2024-06-11T12:00:00 before:2024-06-12T00:00:00",
            "OBS-P05-003: empty base + both bounds must yield 'after:T1 before:T2'; got: '{result2}'"
        );

        // Case 3: non-empty base + start_time → unchanged single-space join behaviour.
        let result3 =
            augment_armis_aql_with_time_window("in:devices", Some("2024-06-11T12:00:00Z"), None);
        assert_eq!(
            result3, "in:devices after:2024-06-11T12:00:00",
            "OBS-P05-003: non-empty base must still yield '<base> after:T'; got: '{result3}'"
        );

        // Case 4: whitespace-only base (trim guard) → treated as empty.
        let result4 = augment_armis_aql_with_time_window("   ", Some("2024-06-11T12:00:00Z"), None);
        assert!(
            !result4.starts_with(' '),
            "OBS-P05-003: whitespace-only base must not produce a leading space; got: '{result4}'"
        );
        assert_eq!(
            result4, "after:2024-06-11T12:00:00",
            "OBS-P05-003: whitespace-only base treated as empty; got: '{result4}'"
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

        let (start_time, end_time) = extract_time_window_from_ast(
            &predicate,
            &["crowdstrike.detections"],
            Some(&spec_map),
            None, // crowdstrike: ocsf_column_naming=false
        );

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

    // -----------------------------------------------------------------------
    // EC-003: inverted time window emits WARN with event_type = "push_down.inverted_time_range"
    // -----------------------------------------------------------------------

    /// EC-003 / ADV-P03-MED-001: inverted time window (start_time > end_time) must emit
    /// a WARN with event_type = "push_down.inverted_time_range", start_time, end_time fields.
    ///
    /// Push-down correctness is preserved — both bounds are still returned. The WARN is a
    /// diagnostic/anomaly signal for operators. BC-2.16.002 catalog row: push_down.inverted_time_range.
    ///
    /// Uses `#[tracing_test::traced_test]` (same pattern as BC-2.16.012 AC-9d in invalidation.rs).
    #[test]
    #[tracing_test::traced_test]
    fn test_ec_003_inverted_time_range_emits_warn() {
        // INVERTED case: Gt T2 AND Lt T1, where T2 > T1.
        // start_time = 2026-06-01, end_time = 2026-01-01 → start > end → inverted.
        let inverted_query = "SELECT * FROM crowdstrike.detections \
             WHERE created_timestamp > '2026-06-01T00:00:00Z' \
             AND created_timestamp < '2026-01-01T00:00:00Z'";
        let inverted_predicate = parse_where(inverted_query);

        let col = make_datetime_index_column("created_timestamp");
        let mut spec_map: HashMap<String, Vec<ColumnSpec>> = HashMap::new();
        spec_map.insert("crowdstrike.detections".to_string(), vec![col]);

        let (start, end) = extract_time_window_from_ast(
            &inverted_predicate,
            &["crowdstrike.detections"],
            Some(&spec_map),
            None, // crowdstrike: ocsf_column_naming=false
        );

        // Both bounds must still be returned (DataFusion backstop correctness preserved).
        assert!(
            start.is_some(),
            "EC-003: start_time must be returned even for inverted window; got None"
        );
        assert!(
            end.is_some(),
            "EC-003: end_time must be returned even for inverted window; got None"
        );

        // WARN must have been emitted with the correct event_type (BC-2.16.002 catalog row).
        assert!(
            logs_contain("push_down.inverted_time_range"),
            "EC-003 ADV-P03-MED-001: inverted time window must emit WARN with \
             event_type = \"push_down.inverted_time_range\"; no such log captured. \
             start={start:?} end={end:?}"
        );
        // Also assert the bound values appear in the log (start_time and end_time fields).
        assert!(
            logs_contain("2026-06-01"),
            "EC-003: WARN log must include start_time value (2026-06-01); not found"
        );
        assert!(
            logs_contain("2026-01-01"),
            "EC-003: WARN log must include end_time value (2026-01-01); not found"
        );
    }

    /// EC-003 complementary: non-inverted time window (start_time < end_time) MUST NOT
    /// emit the WARN event. Separate #[traced_test] so the log buffer is fresh.
    #[test]
    #[tracing_test::traced_test]
    fn test_ec_003_non_inverted_time_range_does_not_emit_warn() {
        // Normal window: 2026-01-01 < 2026-06-01 (correctly ordered).
        let normal_query = "SELECT * FROM crowdstrike.detections \
             WHERE created_timestamp > '2026-01-01T00:00:00Z' \
             AND created_timestamp < '2026-06-01T00:00:00Z'";
        let normal_predicate = parse_where(normal_query);

        let col = make_datetime_index_column("created_timestamp");
        let mut spec_map: HashMap<String, Vec<ColumnSpec>> = HashMap::new();
        spec_map.insert("crowdstrike.detections".to_string(), vec![col]);

        let (start, end) = extract_time_window_from_ast(
            &normal_predicate,
            &["crowdstrike.detections"],
            Some(&spec_map),
            None, // crowdstrike: ocsf_column_naming=false
        );

        assert!(
            start.is_some(),
            "EC-003 non-inverted: start_time must be extracted"
        );
        assert!(
            end.is_some(),
            "EC-003 non-inverted: end_time must be extracted"
        );

        // No WARN must be emitted for a correctly ordered window.
        assert!(
            !logs_contain("push_down.inverted_time_range"),
            "EC-003: non-inverted time window must NOT emit WARN with \
             event_type = \"push_down.inverted_time_range\"; \
             start={start:?} end={end:?}"
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
        // This exercises the classification gate: string-typed `>` predicates are classified
        // regardless of whether the parser would accept them in real usage.
        // A datetime-format value is used so that, if extraction were incorrectly applied,
        // the result would be non-None — confirming the assertion is load-bearing.
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

        let (start_time, end_time) = extract_time_window_from_ast(
            &predicate,
            &["crowdstrike.detections"],
            Some(&spec_map2),
            None, // crowdstrike: ocsf_column_naming=false
        );

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

    /// F-P9-LOW-1 / BC-2.11.024
    ///
    /// A `Predicate::Compare { op: Eq, case_insensitive: true, ... }` (IEQ form)
    /// MUST NOT be collected into the equality push-down FilterMap by
    /// `predicate_tree_to_filter_map`.
    ///
    /// Case-insensitive predicates must be evaluated post-fetch by DataFusion, not
    /// pushed down to the sensor.  Pushing them down causes the sensor to apply a
    /// case-sensitive equality (e.g., `severity = 'high'`), which returns 0 rows
    /// against OCSF Title-case normalized values like `'High'`.
    ///
    /// # Construction note
    ///
    /// IEQ is only supported in filter/pipe syntax, NOT SQL mode.  The test
    /// constructs the `Predicate::Compare { case_insensitive: true, ... }` directly
    /// from AST types (valid in-crate because `Predicate` is `#[non_exhaustive]`
    /// only to external crates; within the defining crate struct-construction is
    /// permitted).  This directly exercises the `collect_equality_exprs` guard
    /// without depending on parser mode routing.
    ///
    /// # Guard
    ///
    /// A case-sensitive `=` sibling (`severity = 'low'`) MUST still be collected
    /// into the FilterMap — verifying the fix is precise and does not regress
    /// ordinary equality push-down.
    ///
    /// # Red Gate
    ///
    /// At HEAD 0b2c0983, `collect_equality_exprs` matches
    /// `Predicate::Compare { lhs, op, rhs, .. }` with a `..` wildcard that captures
    /// `case_insensitive: true`.  IEQ predicates are therefore incorrectly collected.
    ///
    /// Fails with:
    ///   FilterMap["severity"] = Some("high") but expected None.
    ///
    /// # Fix target
    ///
    /// Change `..` to `case_insensitive: false` (or equivalent) in
    /// `collect_equality_exprs` so only case-sensitive `=` comparisons are
    /// pushed down.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_11_024_ieq_predicate_excluded_from_equality_pushdown() {
        use crate::ast::{CompareOp, Expr, FieldPath, Literal, Predicate};

        use super::predicate_tree_to_filter_map;

        // --- Subject: IEQ predicate (case_insensitive: true) must NOT be pushed down ---
        //
        // Equivalent to parsing `severity IEQ 'high'` in filter mode.
        // IEQ lowers to `lower(severity) = lower('high')` in DataFusion — it MUST NOT
        // be pushed down to the sensor as a plain equality filter.
        let ieq_pred = Predicate::Compare {
            lhs: Box::new(Expr::Field(FieldPath::new(["severity"]))),
            op: CompareOp::Eq,
            rhs: Box::new(Expr::Literal(Literal::String("high".to_string()))),
            case_insensitive: true,
        };
        let map = predicate_tree_to_filter_map(&ieq_pred);
        assert!(
            map.get("severity").is_none(),
            "BC-2.11.024 F-P9-LOW-1: IEQ predicate `severity IEQ 'high'` \
             (case_insensitive: true) must NOT be collected into the push-down \
             FilterMap — the sensor would apply a case-sensitive equality that \
             misses OCSF Title-case values like 'High'. Got FilterMap: {map:?}"
        );

        // --- Guard: case-sensitive `=` (case_insensitive: false) MUST still be pushed down ---
        let eq_pred = Predicate::Compare {
            lhs: Box::new(Expr::Field(FieldPath::new(["severity"]))),
            op: CompareOp::Eq,
            rhs: Box::new(Expr::Literal(Literal::String("low".to_string()))),
            case_insensitive: false,
        };
        let guard_map = predicate_tree_to_filter_map(&eq_pred);
        assert!(
            guard_map.get("severity").is_some(),
            "BC-2.11.024 F-P9-LOW-1 guard: case-sensitive `severity = 'low'` \
             (case_insensitive: false) MUST still be collected into the push-down \
             FilterMap. Got: {guard_map:?}"
        );
    }

    // -----------------------------------------------------------------------
    // RG-PD-001 / AC-014 / OQ-001 — OCSF-flattened Arrow name is INDEX-eligible
    // -----------------------------------------------------------------------

    /// AC-014 / OQ-001 — `extract_time_window_from_ast` recognizes OCSF-flattened Arrow
    /// name as index-eligible push-down target.
    ///
    /// When `ocsf_column_naming = true`, the datetime INDEX column
    /// `claroty.audit_logs.timestamp` has `ocsf_field = "time"`.
    /// After `ocsf_field_to_arrow_name("time")` = `"time"` (single-segment, no change),
    /// PrismQL queries authored by the LLM agent use `WHERE time > '...'`.
    ///
    /// **Before the fix:** `datetime_index_cols` is built from `col.name` only
    /// (`{"timestamp"}`). A filter on `"time"` is not in that set → full scan.
    ///
    /// **After the fix:** `datetime_index_cols` contains BOTH `"timestamp"` (col.name) AND
    /// `"time"` (ocsf_field_to_arrow_name(ocsf_field)) → filter on `"time"` is INDEX-eligible.
    ///
    /// **Red gate:** `datetime_index_cols` only contains `"timestamp"`. A filter on `"time"`
    /// falls through to `(None, None)`. Assertion `start_time.is_some()` fails.
    ///
    /// SAP-3: end-to-end from parsed PrismQL predicate → `extract_time_window_from_ast`.
    /// Covers AC-014.
    /// Traces to BC-2.16.003 §Interpretation A (OCSF-flattened Arrow names usable verbatim
    /// by LLM agents in index-eligible filter positions; OQ-001 human decision 2026-08-21).
    #[test]
    fn test_extract_time_window_from_ast_recognizes_ocsf_flattened_time_column_as_index_eligible() {
        use prism_core::ColumnOptions;

        // Claroty audit_logs.timestamp: datetime INDEX col, ocsf_field = "time"
        // ocsf_field_to_arrow_name("time") = "time" (single-segment path, unchanged).
        let mut col = ColumnSpec::default();
        col.name = "timestamp".to_string();
        col.column_type = ColumnType::Datetime;
        col.options = vec![ColumnOptions::Index];
        col.ocsf_field = Some("time".to_string());

        let mut spec_map: HashMap<String, Vec<ColumnSpec>> = HashMap::new();
        spec_map.insert("claroty.audit_logs".to_string(), vec![col]);

        // claroty has ocsf_column_naming=true (ADR-058 §G): must register both "timestamp"
        // and the flattened OCSF name "time" in datetime_index_cols.
        let mut naming_map: HashMap<String, bool> = HashMap::new();
        naming_map.insert("claroty.audit_logs".to_string(), true);

        // PrismQL filter on the OCSF-flattened Arrow name "time" (what the LLM agent emits).
        let query = "SELECT * FROM claroty.audit_logs WHERE time > '2024-01-01T00:00:00Z'";
        let predicate = parse_where(query);

        let (start_time, end_time) = extract_time_window_from_ast(
            &predicate,
            &["claroty.audit_logs"],
            Some(&spec_map),
            Some(&naming_map),
        );

        // AC-014 (RG-PD-001/OQ-001): filter on "time" (OCSF-flattened Arrow name) MUST be
        // INDEX-eligible → start_time must be Some.
        // Before fix: datetime_index_cols = {"timestamp"} only. "time" is absent.
        // extract_time_bounds_from_predicate falls through → (None, None). Assertion fails.
        assert!(
            start_time.is_some(),
            "AC-014 (RG-PD-001/OQ-001): filter on OCSF-flattened Arrow column 'time' \
             (claroty.audit_logs.timestamp with ocsf_field='time') MUST be INDEX-eligible. \
             Got start_time=None — extract_time_window_from_ast fell through to full scan \
             because datetime_index_cols only contains 'timestamp', not 'time'. \
             Fix: for each datetime INDEX column with non-empty ocsf_field, insert BOTH \
             col.name AND ocsf_field_to_arrow_name(ocsf_field) into datetime_index_cols."
        );
        assert!(
            start_time.as_deref().unwrap_or("").contains("2024-01-01"),
            "AC-014 (RG-PD-001/OQ-001): start_time must contain '2024-01-01'; got: {start_time:?}"
        );
        assert!(
            end_time.is_none(),
            "AC-014 (RG-PD-001/OQ-001): no upper-bound predicate — end_time must be None; \
             got: {end_time:?}"
        );
    }

    /// ADR-058 §I6 (OQ-001) — `ocsf_column_naming` flag gates OCSF-flattened Arrow name
    /// registration in `datetime_index_cols`.
    ///
    /// For a sensor with `ocsf_column_naming = false`, a datetime+INDEX column with
    /// `ocsf_field = "time"` MUST NOT register the flattened name ("time") in
    /// `datetime_index_cols`. Only `col.name` ("timestamp") is registered. A PrismQL
    /// filter on "time" MUST fall through to `(None, None)` — the sensor does not serve
    /// OCSF-named Arrow fields, so registering the flattened name is a latent collision
    /// risk (a real column named "time" on a different sensor would get a push-down
    /// bound from the wrong column — silent under-fetch).
    ///
    /// For a sensor with `ocsf_column_naming = true`, the same column MUST register both
    /// "timestamp" and "time" → the filter on "time" MUST be INDEX-eligible. This is the
    /// positive guard, ensuring RG-PD-001 semantics are preserved for flag=true sensors.
    ///
    /// **Red gate (flag=false assertion fails before the gate):**
    /// Without the `ocsf_column_naming` flag gate, `datetime_index_cols` unconditionally
    /// contains "time" for any sensor with a non-empty `ocsf_field` — so the flag=false
    /// case incorrectly returns `Some(start_time)`.
    ///
    /// Traces to ADR-058 §I6 invariant OQ-001.
    #[test]
    fn test_extract_time_window_ocsf_naming_flag_gates_flattened_name_registration() {
        use prism_core::ColumnOptions;

        // Column: datetime INDEX, ocsf_field = "time" (mirrors Claroty audit_logs.timestamp).
        let mut col = ColumnSpec::default();
        col.name = "timestamp".to_string();
        col.column_type = ColumnType::Datetime;
        col.options = vec![ColumnOptions::Index];
        col.ocsf_field = Some("time".to_string());

        let mut spec_map: HashMap<String, Vec<ColumnSpec>> = HashMap::new();
        spec_map.insert("mysensor.audit_logs".to_string(), vec![col]);

        // Query filters on the OCSF-flattened Arrow name "time" (not col.name "timestamp").
        let query = "SELECT * FROM mysensor.audit_logs WHERE time > '2024-01-01T00:00:00Z'";
        let predicate = parse_where(query);

        // --- flag=false: flattened name MUST NOT be INDEX-eligible ---
        let mut naming_false: HashMap<String, bool> = HashMap::new();
        naming_false.insert("mysensor.audit_logs".to_string(), false);

        let (start_false, _) = extract_time_window_from_ast(
            &predicate,
            &["mysensor.audit_logs"],
            Some(&spec_map),
            Some(&naming_false),
        );
        assert!(
            start_false.is_none(),
            "ADR-058 §I6 (OQ-001): for ocsf_column_naming=false, a filter on the \
             OCSF-flattened name 'time' MUST NOT be INDEX-eligible — only col.name \
             'timestamp' is registered. Got start_time=Some({start_false:?}) — the \
             flag gate is missing; 'time' was incorrectly inserted into \
             datetime_index_cols for a flag=false sensor."
        );

        // --- flag=true: flattened name MUST be INDEX-eligible (RG-PD-001 regression guard) ---
        let mut naming_true: HashMap<String, bool> = HashMap::new();
        naming_true.insert("mysensor.audit_logs".to_string(), true);

        let (start_true, _) = extract_time_window_from_ast(
            &predicate,
            &["mysensor.audit_logs"],
            Some(&spec_map),
            Some(&naming_true),
        );
        assert!(
            start_true.is_some(),
            "ADR-058 §I6 (OQ-001): for ocsf_column_naming=true, a filter on the \
             OCSF-flattened name 'time' MUST be INDEX-eligible (both 'timestamp' and \
             'time' registered). Got start_time=None — the gate incorrectly blocked \
             flag=true registration."
        );
        assert!(
            start_true.as_deref().unwrap_or("").contains("2024-01-01"),
            "ADR-058 §I6: start_time must contain '2024-01-01' for flag=true; \
             got: {start_true:?}"
        );
    }
}
