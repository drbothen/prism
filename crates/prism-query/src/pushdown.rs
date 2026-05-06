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

// S-3.02 stub functions: dead_code suppressed for stub phase (BC-5.38.001).
#![allow(dead_code)]

use prism_sensors::SensorSpec;

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
pub fn classify_predicates(_where_clause: &[Expr], _source: &SensorSpec) -> PushDownPlan {
    todo!("S-3.02 — classify_predicates")
}

// ---------------------------------------------------------------------------
// column_push_down_option
// ---------------------------------------------------------------------------

/// Determine the push-down option for a given column on a sensor spec.
///
/// Returns `ColumnPushDownOption::Default` when the column is not declared by
/// the sensor spec (conservative fallback). (BC-2.11.007)
pub(crate) fn column_push_down_option(
    _column_name: &str,
    _source: &SensorSpec,
) -> ColumnPushDownOption {
    todo!("S-3.02 — column_push_down_option")
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
pub(crate) fn translate_push_down_filter(
    _predicate: &Predicate,
    _source: &SensorSpec,
) -> Option<String> {
    todo!("S-3.02 — translate_push_down_filter")
}
