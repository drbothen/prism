//! `explain` — PrismQL query plan analysis without execution.
//!
//! Implements BC-2.11.010: parse and plan a PrismQL query without issuing any
//! sensor API calls. Returns a structured `ExplainResult` containing the
//! detected query mode, alias expansions, field resolution, per-sensor push-down
//! filters as PrismQL-native predicate strings (sensor-native translation deferred
//! to S-3.X via TD-S303-PUSH-DOWN-TRANSLATION-001), AST-derived post-fetch
//! operations, and a cost estimate.
//!
//! # Architecture Compliance (BC-2.11.010)
//! - MUST NOT call `fan_out()` or any sensor adapter `fetch()` method.
//! - Reuses `classify_predicates()` from `pushdown.rs` — do NOT duplicate logic.
//! - Reuses `resolve_clients()` from `scoping.rs`.
//! - DataFusion logical plan is elided; `post_fetch_operations` are AST-derived
//!   (TD-S303-DATAFUSION-PLAN-001). Do NOT invoke `SessionContext::create_logical_plan()`.
//! - An audit event MUST be emitted for every invocation (DI-004).
//! - Syntactic security limits apply; materialization limit produces a warning,
//!   not an error (DI-019).
//!
//! # BC References
//! - BC-2.11.010 — `explain_query` MCP Tool
//! - BC-2.11.007 — Sensor Filter Push-Down (reused push-down classification)
//! - BC-2.11.011 — Cross-Client Query Scoping (reused resolve_clients)
//!
//! Story: S-3.03

use std::collections::HashMap;
use std::sync::Arc;

use prism_core::{OrgSlug, PrismError, SensorId};
use serde::Serialize;

use crate::ast::{Ast, SourceRef, SourceRefKind, SqlStatement, VirtualField};
use crate::filter_parser::PrismQlParser;
use crate::pushdown::classify_predicates;
use crate::scoping::{resolve_clients, ClientRegistry};
use crate::security::PRISM_MAX_QUERY_SIZE;
use crate::visit::{walk_ast, Visitor};

// ---------------------------------------------------------------------------
// AuditEvent (DI-004)
// ---------------------------------------------------------------------------

/// Lightweight audit event emitted for every `explain()` invocation (DI-004).
///
/// Emitted via `ExplainOptions::audit_sink` when provided. At the MCP layer,
/// `prism-mcp` routes these through the full `AuditEmitterService`; in tests
/// the sink captures the event for assertion.
///
/// The audit event is emitted for BOTH success and error paths (DI-004).
#[derive(Debug, Clone)]
pub struct AuditEvent {
    /// The raw query string as provided.
    pub query: String,
    /// The client scope parameter.
    pub clients: Option<Vec<OrgSlug>>,
    /// The sensor scope parameter.
    pub sensors: Option<Vec<SensorId>>,
    /// The source scope parameter.
    pub sources: Option<Vec<String>>,
    /// Human-readable outcome summary (e.g. "success", "E-QUERY-001").
    pub outcome_summary: String,
}

// CR-009: Compile-time assertion that AuditEvent is Send + Sync.
// AuditEvent crosses thread boundaries when the audit_sink Arc is shared.
// Uses a trait-bound fn-pointer cast (zero-cost, works on stable Rust).
fn _assert_audit_event_send_sync()
where
    AuditEvent: Send + Sync,
{
}

// ---------------------------------------------------------------------------
// ExplainOptions
// ---------------------------------------------------------------------------

/// Per-call options for the `explain_query` MCP tool.
///
/// Mirrors the scoping parameters of the `query` tool (BC-2.11.010 Preconditions).
#[derive(Clone, Default)]
pub struct ExplainOptions {
    /// Client scope override: `None` = all configured clients. (BC-2.11.011)
    pub clients: Option<Vec<OrgSlug>>,
    /// Sensor scope override: `None` = all sensors for resolved clients. (BC-2.11.010)
    pub sensors: Option<Vec<SensorId>>,
    /// Data source scope override: `None` = all sources for resolved sensors. (BC-2.11.010)
    pub sources: Option<Vec<String>>,
    /// Alias registry mapping alias names to their expanded definitions.
    ///
    /// Used to expand alias references in the query and populate `alias_expansion`
    /// in the result. Until S-3.04 (alias registry story) merges, callers must
    /// provide this at call time. (BC-2.11.010 postcondition `alias_expansion`)
    ///
    /// Callers should pass only aliases relevant to context; the config layer
    /// enforces a maximum alias count per DI-028. (SEC-003)
    pub alias_registry: HashMap<String, String>,
    /// Client registry for resolving `clients: None` to all configured clients.
    ///
    /// `None` uses an empty registry (no configured clients).
    pub client_registry: Option<Arc<ClientRegistry>>,
    /// DI-004 audit sink. Called once for every invocation (success AND error).
    ///
    /// In production, `prism-mcp` provides the real audit emitter. Tests provide
    /// a capturing closure.
    ///
    /// Note: `Clone` on `ExplainOptions` shares the same sink `Arc`
    /// (reference-counted). Both the original and clone call the same sink
    /// closure. (CR-013)
    ///
    // TODO(CR-007): replace with `AuditEmitter` trait from `prism-audit` when
    // wired (S-X.XX). The current `Arc<dyn Fn>` is a lightweight stand-in.
    pub audit_sink: Option<Arc<dyn Fn(AuditEvent) + Send + Sync>>,
}

impl std::fmt::Debug for ExplainOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExplainOptions")
            .field("clients", &self.clients)
            .field("sensors", &self.sensors)
            .field("sources", &self.sources)
            .field("alias_registry", &self.alias_registry)
            .field("audit_sink", &self.audit_sink.as_ref().map(|_| "<sink>"))
            .finish()
    }
}

// ---------------------------------------------------------------------------
// ExplainResult (BC-2.11.010 Postconditions)
// ---------------------------------------------------------------------------

/// The output of a successful `explain_query` invocation.
///
/// Implements BC-2.11.010 postconditions. All fields are JSON-serializable
/// using standard types — no custom serializer required (Story §Architecture
/// Compliance Rules).
///
/// # Field Name Notes (CR-004)
/// Field names here follow BC-2.11.010 postconditions. The S-3.03 story spec
/// used slightly different names in some places (e.g., `push_down_predicates` →
/// `api_filters_pushed`, `sources` → `sensors_to_query`). The BC postconditions
/// are authoritative; these field names are stable API.
// SEC-004: Deserialize removed from output types — ExplainResult is write-only
// (produced by the engine, serialized to JSON for the caller). No deserialization
// path is needed in production; gating Deserialize behind #[cfg(test)] would be
// premature optimization since no test currently round-trips this type.
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
pub struct ExplainResult {
    /// The detected query mode: `"filter"`, `"sql"`, or `"pipe"`. (BC-2.11.010)
    pub parsed_mode: String,

    /// The raw query string as provided by the caller. (BC-2.11.010)
    pub original_query: String,

    /// Map of alias names to their expanded definitions. Empty if no aliases
    /// were used. (BC-2.11.010)
    pub alias_expansion: HashMap<String, String>,

    /// The query after all alias expansion. (BC-2.11.010)
    pub expanded_query: String,

    /// Map of field names used in the query to their OCSF paths and how they
    /// were resolved (direct, alias, or virtual). (BC-2.11.010)
    pub field_resolution: HashMap<String, FieldResolution>,

    /// The execution plan showing sensors, push-down filters, and post-fetch
    /// operations. (BC-2.11.010)
    pub execution_plan: ExecutionPlan,

    /// Structured cost estimate for the query. (BC-2.11.010)
    pub estimated_cost: CostEstimate,
}

// ---------------------------------------------------------------------------
// FieldResolution
// ---------------------------------------------------------------------------

/// How a single field name in the query was resolved to an OCSF path.
///
/// Part of `ExplainResult.field_resolution`. (BC-2.11.010)
// SEC-004: Deserialize removed — output-only type.
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
pub struct FieldResolution {
    /// OCSF path the field maps to (e.g., `"finding.severity_id"`).
    pub ocsf_path: String,
    /// Resolution method: `"direct"`, `"alias"`, or `"virtual"`.
    pub resolution_method: String,
}

// ---------------------------------------------------------------------------
// ExecutionPlan
// ---------------------------------------------------------------------------

/// The execution plan produced without running the query. (BC-2.11.010)
// SEC-004: Deserialize removed — output-only type.
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
pub struct ExecutionPlan {
    /// List of sensors that would be queried. (BC-2.11.010)
    pub sensors_to_query: Vec<ExplainSource>,

    /// Per-source post-fetch operations (filter, group-by, sort, limit, etc.).
    /// (BC-2.11.010)
    pub post_fetch_operations: Vec<String>,

    /// All client IDs that would be in scope for this query.
    ///
    /// When `ExplainOptions::clients` is `None`, this lists all configured
    /// clients from the registry (AC-5, BC-2.11.010 / BC-2.11.011). No
    /// fan-out or sensor API calls occur — the list is resolved from the
    /// `ClientRegistry` only.
    pub clients_to_query: Vec<OrgSlug>,
}

// ---------------------------------------------------------------------------
// ExplainSource
// ---------------------------------------------------------------------------

/// Per-sensor push-down information for the explain result. (BC-2.11.010)
// SEC-004: Deserialize removed — output-only type.
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
pub struct ExplainSource {
    /// Source reference string, e.g. `"crowdstrike.detections"`.
    pub source_ref: String,

    /// The sensor id this source belongs to.
    pub sensor_type: SensorId,

    /// Push-down predicates as PrismQL-native predicate strings (e.g. `"severity = 'critical'"`).
    /// Sensor-native translation (FQL, KQL, etc.) is deferred to S-3.X via
    /// TD-S303-PUSH-DOWN-TRANSLATION-001. (BC-2.11.010 `api_filters_pushed` / INV-PUSH-001)
    pub api_filters_pushed: Vec<String>,

    /// Predicates applied post-materialization (not pushed to sensor API). (BC-2.11.010)
    pub post_filter_predicates: Vec<String>,

    /// Estimated row count from sensor count hint, if available.
    /// `None` if the sensor adapter does not expose a count hint. (Story §Dev Notes)
    pub estimated_row_count: Option<u64>,
}

// ---------------------------------------------------------------------------
// CostEstimate
// ---------------------------------------------------------------------------

/// Structured cost estimate for the query. (BC-2.11.010 `estimated_cost`)
// SEC-004: Deserialize removed — output-only type.
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
pub struct CostEstimate {
    /// Per-sensor estimated latency based on rolling historical averages.
    /// Map from sensor identifier to estimated latency in milliseconds.
    pub per_sensor_latency_ms: HashMap<String, u64>,

    /// Estimated API call count per sensor based on expected pagination depth.
    pub per_sensor_api_call_count: HashMap<String, u64>,

    /// Rate limit headroom per sensor (remaining calls in the current window).
    pub per_sensor_rate_limit_headroom: HashMap<String, u64>,

    /// Human-readable summary combining latency, call count, and rate limit
    /// into an actionable estimate for the analyst or AI agent.
    pub summary: String,

    /// Warnings attached to this estimate. Non-empty when, e.g., the estimated
    /// record count exceeds the 10K materialization limit (EC-11-025, DI-019).
    pub warnings: Vec<String>,
}

// ---------------------------------------------------------------------------
// FieldCollector — AST visitor for extracting field references
// ---------------------------------------------------------------------------

/// Collects all field names and virtual fields referenced in an AST.
struct FieldCollector {
    /// Regular field names found in the AST.
    fields: Vec<String>,
    /// Virtual field names found in the AST.
    virtual_fields: Vec<String>,
}

impl FieldCollector {
    fn new() -> Self {
        Self {
            fields: Vec::new(),
            virtual_fields: Vec::new(),
        }
    }
}

impl Visitor for FieldCollector {
    fn visit_field(&mut self, f: &crate::ast::FieldPath) {
        let name = f.segments.join(".");
        if !self.fields.contains(&name) {
            self.fields.push(name);
        }
    }

    fn visit_virtual_field(&mut self, vf: &VirtualField) {
        let name = virtual_field_name(vf).to_string();
        if !self.virtual_fields.contains(&name) {
            self.virtual_fields.push(name);
        }
    }

    // Override source-containing walk functions to skip the source SourceRef
    // `as_field_path()` call that would otherwise collect table names
    // (e.g. "crowdstrike.detections") into field_resolution. (CR-001)

    fn visit_filter_expr(&mut self, fe: &crate::ast::FilterExpr) {
        // Skip fe.source — it is a SourceRef, not a query field.
        //
        // SEC-P8-001: FilterExpr is #[non_exhaustive]. Current fields: source, predicate.
        // If a future field with FieldPath or Predicate references is added, this
        // override MUST be updated to walk it — otherwise field_resolution will
        // silently miss those fields. (Mirrors visit_join / SEC-P7-001 and
        // visit_join_stage / SEC-P3-002.)
        self.visit_predicate(&fe.predicate);
    }

    fn visit_join(&mut self, j: &crate::ast::Join) {
        // Skip j.source — it is a SourceRef (e.g. "crowdstrike.events"), not a
        // query field. walk_join() calls visit_field(&j.source.as_field_path())
        // which would leak the JOIN target table name into field_resolution.
        // Only visit the join ON expression for field collection.
        //
        // SEC-P7-001: Join is #[non_exhaustive]. Current fields: kind, source, alias, on.
        // If a future field with FieldPath or Predicate references is added, this
        // override MUST be updated to walk it — otherwise field_resolution will
        // silently miss those fields. (Mirrors visit_join_stage / SEC-P3-002.)
        self.visit_expr(&j.on);
    }

    fn visit_sql_query(&mut self, q: &crate::ast::SqlQuery) {
        // Skip q.from.source — it is a SourceRef, not a query field.
        //
        // SEC-P9-001: SqlQuery is #[non_exhaustive]. Current fields: select, from,
        // joins (walked via visit_join), where_, group_by, having, order_by, limit.
        // If a future field with FieldPath or Predicate references is added (e.g.,
        // a WITH/CTE clause per ast.rs:100 rationale), this override MUST be updated
        // to walk it — otherwise field_resolution will silently miss those fields.
        // (Mirrors visit_filter_expr / SEC-P8-001, visit_join / SEC-P7-001,
        // visit_pipe_query / SEC-P8-002, visit_join_stage / SEC-P3-002.)
        self.visit_select_clause(&q.select);
        for join in &q.joins {
            // Delegate to visit_join so the source-ref skip logic is centralised
            // and any future callers of visit_join also get the correct behaviour.
            self.visit_join(join);
        }
        if let Some(pred) = &q.where_ {
            self.visit_predicate(pred);
        }
        for expr in &q.group_by {
            self.visit_expr(expr);
        }
        if let Some(pred) = &q.having {
            self.visit_predicate(pred);
        }
        for oe in &q.order_by {
            self.visit_order_expr(oe);
        }
    }

    fn visit_pipe_query(&mut self, q: &crate::ast::PipeQuery) {
        // Skip q.source — it is a SourceRef, not a query field.
        //
        // SEC-P8-002: PipeQuery is #[non_exhaustive]. Current fields: source (skipped),
        // stages (walked), write (walked). All non-source fields are visited completely.
        // If a future field with FieldPath or Predicate references is added, this
        // override MUST be updated to walk it — otherwise field_resolution will
        // silently miss those fields. (Mirrors visit_join / SEC-P7-001 and
        // visit_join_stage / SEC-P3-002.)
        for stage in &q.stages {
            self.visit_pipe_stage(stage);
        }
        if let Some(write) = &q.write {
            self.visit_write_node(write);
        }
    }

    fn visit_join_stage(&mut self, js: &crate::ast::JoinStage) {
        // CR-016: Skip js.source — it is a SourceRef (e.g. "armis.devices"),
        // not a query field. walk_join_stage() calls visit_field(&js.source.as_field_path())
        // which would leak the JOIN target table name into field_resolution.
        // Only visit the join condition fields.
        //
        // SEC-P3-002: JoinStage is #[non_exhaustive]. Current fields: kind, source, on.
        // If a future field with FieldPath references is added (e.g., a join filter
        // predicate), this override MUST be updated to walk it. The catch-all `_ => {}`
        // in the JoinCondition match below handles future variants of that enum, but
        // does NOT address new top-level JoinStage fields.
        use crate::ast::JoinCondition;
        // `JoinCondition` is `#[non_exhaustive]`; `_ => {}` is required for
        // forward compatibility with future variants.
        #[allow(unreachable_patterns)]
        match &js.on {
            JoinCondition::SameField(f) => self.visit_field(f),
            JoinCondition::Pair(l, r) => {
                self.visit_field(l);
                self.visit_field(r);
            }
            _ => {}
        }
    }
}

/// Map a `VirtualField` to its string identifier.
#[allow(unreachable_patterns)]
fn virtual_field_name(vf: &VirtualField) -> &'static str {
    match vf {
        VirtualField::Sensor => "_sensor",
        VirtualField::Client => "_client",
        VirtualField::SourceTable => "_source_table",
        VirtualField::SourceType => "_source_type",
        VirtualField::SafetyFlags => "_safety_flags",
        _ => "_unknown",
    }
}

// ---------------------------------------------------------------------------
// has_or_predicate — detect OR nodes in a predicate tree
// ---------------------------------------------------------------------------

/// Return `true` if the AST contains any WHERE clause with a `Predicate::Logical { op: Or, .. }`.
///
/// Used to emit a warning in the cost estimate (I-LOCAL-004): OR predicates are
/// flattened during push-down classification, which may mislead callers about
/// the actual push-down semantics. The warning prompts awareness; S-3.X refines
/// OR-mode handling with sensor-native OR translation.
fn has_or_predicate(ast: &Ast) -> bool {
    use crate::ast::{LogicalOp, Predicate, SqlStatement};

    fn pred_has_or(pred: &Predicate) -> bool {
        match pred {
            Predicate::Logical {
                op: LogicalOp::Or, ..
            } => true,
            Predicate::Logical { predicates, .. } => predicates.iter().any(pred_has_or),
            Predicate::Not(inner) => pred_has_or(inner),
            _ => false,
        }
    }

    match ast {
        Ast::Filter(fe) => pred_has_or(&fe.predicate),
        Ast::Sql(SqlStatement::Select(sq)) => sq.where_.as_ref().is_some_and(pred_has_or),
        Ast::Pipe(pq) => pq.stages.iter().any(|stage| {
            if let crate::ast::PipeStage::Where(pred) = stage {
                pred_has_or(pred)
            } else {
                false
            }
        }),
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// extract_sources_from_ast — collect SourceRef entries directly from AST
// ---------------------------------------------------------------------------

/// Extract all source references from an AST, deduplicated by raw string.
///
/// The visitor trait does not expose `visit_source_ref` because `SourceRef`
/// is embedded structurally in the AST (not traversed by `walk_*`). We
/// access sources directly from the top-level AST variants. (BC-2.11.010)
fn extract_sources_from_ast(ast: &Ast) -> Vec<SourceRef> {
    let mut sources: Vec<SourceRef> = Vec::new();

    // Dedup-push helper — avoids duplicate entries by raw string.
    fn push_dedup(sources: &mut Vec<SourceRef>, s: &SourceRef) {
        if !sources.iter().any(|x| x.raw == s.raw) {
            sources.push(s.clone());
        }
    }

    match ast {
        Ast::Filter(fe) => {
            push_dedup(&mut sources, &fe.source);
        }
        Ast::Sql(SqlStatement::Select(sq)) => {
            push_dedup(&mut sources, &sq.from.source);
            for join in &sq.joins {
                push_dedup(&mut sources, &join.source);
            }
        }
        Ast::Sql(SqlStatement::Dml(dml)) => {
            // F-LP6-LOW-1: DML carries source_select (INSERT … SELECT …) and filter
            // (UPDATE/DELETE WHERE) — both can reference internal tables via subqueries.
            // EXPLAIN output's "sensors_to_query" must reflect DML source_select sources
            // so operators see the full query plan for DML queries.
            // Layer 1 sibling-pattern lineage: F-LP3-CRIT-1 → F-LP4-MED-1 → F-LP5-LOW-1 → F-LP6-LOW-1.
            if let Some(ref source_select) = dml.source_select {
                push_dedup(&mut sources, &source_select.from.source);
                for join in &source_select.joins {
                    push_dedup(&mut sources, &join.source);
                }
            }
            if let Some(ref filter) = dml.filter {
                // Walk predicate for InSubquery sources (e.g. DELETE WHERE x IN (SELECT … FROM prism_audit)).
                collect_predicate_sources_into(filter, &mut sources);
            }
        }
        Ast::Pipe(pq) => {
            push_dedup(&mut sources, &pq.source);
            // C-LOCAL-001: also collect JOIN stage sources so that
            // `crowdstrike.devices | join armis.devices on hostname`
            // correctly reports both sensors in `sensors_to_query`.
            for stage in &pq.stages {
                if let crate::ast::PipeStage::Join(js) = stage {
                    push_dedup(&mut sources, &js.source);
                }
            }
        }
        // SqlStatement and Ast are #[non_exhaustive]; wildcard required for future variants.
        #[allow(unreachable_patterns)]
        _ => {}
    }

    sources
}

/// Walk a `Predicate` tree and collect `SourceRef`s from any `InSubquery` predicates.
///
/// Used by `extract_sources_from_ast` for the DML filter arm (F-LP6-LOW-1): a DML
/// WHERE clause may contain `field IN (SELECT … FROM <source>)` which references
/// an internal table. This function recursively extracts those source refs so that
/// EXPLAIN correctly reports all sensors_to_query for DML statements.
fn collect_predicate_sources_into(predicate: &crate::ast::Predicate, sources: &mut Vec<SourceRef>) {
    use crate::ast::Predicate;

    fn push_dedup(sources: &mut Vec<SourceRef>, s: &SourceRef) {
        if !sources.iter().any(|x| x.raw == s.raw) {
            sources.push(s.clone());
        }
    }

    match predicate {
        Predicate::InSubquery { subquery, .. } => {
            push_dedup(sources, &subquery.from.source);
            for join in &subquery.joins {
                push_dedup(sources, &join.source);
            }
        }
        Predicate::Logical { predicates, .. } => {
            for p in predicates {
                collect_predicate_sources_into(p, sources);
            }
        }
        Predicate::Not(inner) => {
            collect_predicate_sources_into(inner, sources);
        }
        // Other predicate variants (Compare, Between, Cidr, Has, Missing, IsNull,
        // Wildcard, RecoveryError) do not carry nested SqlQuery references.
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// PostFetchOperationExtractor — derives post-fetch operations from AST
// ---------------------------------------------------------------------------

/// Derive the list of post-fetch operations (GROUP BY, SORT BY, LIMIT, etc.)
/// from the AST. These are operations that cannot be pushed to sensor APIs and
/// must be applied post-materialization. Derived from AST only — DataFusion
/// plan generation is elided per TD-S303-DATAFUSION-PLAN-001. (BC-2.11.010)
fn post_fetch_operations_from_ast(ast: &Ast) -> Vec<String> {
    let mut ops: Vec<String> = Vec::new();

    match ast {
        Ast::Sql(sql_stmt) => {
            use crate::ast::SqlStatement;
            if let SqlStatement::Select(ref sq) = sql_stmt {
                if !sq.group_by.is_empty() {
                    let cols: Vec<String> = sq.group_by.iter().map(|e| format!("{e:?}")).collect();
                    ops.push(format!("GROUP BY {} column(s)", cols.len()));
                }
                if !sq.order_by.is_empty() {
                    ops.push(format!("SORT BY {} column(s)", sq.order_by.len()));
                }
                if let Some(limit) = sq.limit {
                    ops.push(format!("LIMIT {limit}"));
                }
                if sq.where_.is_some() {
                    // TODO(CR-010, S-3.X): only emit when at least one predicate is
                    // post_filter (not pushed down). Requires threading the push-down
                    // plan into post_fetch_operations_from_ast.
                    ops.push("WHERE filter (post-materialization)".to_string());
                }
            }
        }
        Ast::Pipe(pq) => {
            for stage in &pq.stages {
                use crate::ast::PipeStage;
                match stage {
                    PipeStage::Where(_) => {
                        ops.push("WHERE filter (post-materialization)".to_string());
                    }
                    PipeStage::Sort(exprs) => {
                        ops.push(format!("SORT BY {} column(s)", exprs.len()));
                    }
                    PipeStage::Limit(n) => {
                        ops.push(format!("LIMIT {n}"));
                    }
                    PipeStage::Tail(n) => {
                        ops.push(format!("TAIL {n}"));
                    }
                    PipeStage::Stats(ss) => {
                        ops.push(format!(
                            "GROUP BY {} column(s), {} aggregate(s)",
                            ss.by_fields.len(),
                            ss.aggregates.len()
                        ));
                    }
                    PipeStage::Dedup(fields) => {
                        ops.push(format!("DEDUP {} column(s)", fields.len()));
                    }
                    PipeStage::Fields(_) => {
                        ops.push("FIELDS projection".to_string());
                    }
                    PipeStage::Join(_) => {
                        ops.push("JOIN".to_string());
                    }
                    PipeStage::Enrich(_) => {
                        ops.push("ENRICH".to_string());
                    }
                    // #[non_exhaustive] catch-all for future pipe stages.
                    #[allow(unreachable_patterns)]
                    _ => {}
                }
            }
        }
        Ast::Filter(fe) => {
            // Filter queries: the predicate is the only post-fetch operation
            // (assuming no push-down capable columns; if push-down applies,
            // that predicate goes to api_filters_pushed per-source).
            let _ = fe; // predicate classification is per-source via classify_predicates
                        // TODO(CR-010, S-3.X): only emit post-fetch ops when the push-down plan
                        // confirms at least one predicate is post_filter (not api_filters_pushed).
                        // Requires threading the classify_predicates plan result into this function.
        }
    }

    ops
}

// ---------------------------------------------------------------------------
// sensor_type_from_source_ref — derive SensorId from SourceRef
// ---------------------------------------------------------------------------

/// Derive the sensor id for a source reference.
///
/// Returns `None` for composite, internal, or custom source kinds that do not
/// map to a specific sensor adapter. Any non-empty external sensor name is valid
/// (open dispatch — no closed-enum match).
fn sensor_type_from_source_ref(s: &SourceRef) -> Option<SensorId> {
    match &s.kind {
        SourceRefKind::External { sensor, .. } => {
            let lower = sensor.to_lowercase();
            if lower.is_empty() {
                None
            } else {
                Some(SensorId::from(lower.as_str()))
            }
        }
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// predicates_from_ast — extract WHERE predicates from an AST as Expr slices
// ---------------------------------------------------------------------------

/// Extract the WHERE predicate expressions from the AST for push-down
/// classification. Returns a list of `Expr` nodes that can be classified via
/// `classify_predicates`. (BC-2.11.007, reused per BC-2.11.010)
fn predicates_from_ast(ast: &Ast) -> Vec<crate::ast::Expr> {
    use crate::ast::{Expr, Predicate, SqlStatement};

    fn predicate_to_exprs(pred: &Predicate) -> Vec<Expr> {
        // Convert predicate tree to a flat list of Compare expressions
        // so classify_predicates can see each column-level constraint.
        match pred {
            Predicate::Compare { lhs, op, rhs } => {
                vec![Expr::Compare {
                    lhs: lhs.clone(),
                    op: op.clone(),
                    rhs: rhs.clone(),
                }]
            }
            Predicate::Logical { op, predicates } => {
                let mut out = Vec::new();
                for p in predicates {
                    // For logical combos, flatten into top-level exprs
                    // so each clause can be independently classified.
                    out.extend(predicate_to_exprs(p));
                    let _ = op; // op direction doesn't affect push-down eligibility
                }
                out
            }
            // CR-003: NOT semantics cannot be safely push-down classified without
            // sensor-native NOT translation (FQL does not support arbitrary NOT).
            // Conservatively return empty — NOT predicates fall to post_filter.
            // TODO(S-3.X): implement sensor-native NOT translation for push-down eligibility.
            //
            // I-LOCAL-NEW-1 exception: when NOT wraps a CIDR predicate specifically,
            // emit a NotCidr Compare so predicate_as_string can render it with the
            // "NOT " prefix.  This preserves the semantic distinction between
            // "src_ip CIDR '10.0.0.0/8'" and "NOT src_ip CIDR '10.0.0.0/8'" in the
            // explain output, avoiding silent identity of negated and non-negated forms.
            // Push-down is still disallowed (NotCidr is never pushed to api_filters_pushed
            // because the ColumnSpec lookup conservatively routes it to post_filter).
            Predicate::Not(inner) => match inner.as_ref() {
                Predicate::Cidr { field, cidr, .. } => vec![Expr::Compare {
                    lhs: Box::new(Expr::Field(field.clone())),
                    op: crate::ast::CompareOp::NotCidr,
                    rhs: Box::new(Expr::Literal(crate::ast::Literal::Cidr(cidr.clone()))),
                }],
                _ => vec![],
            },
            Predicate::StringOp { field, .. } => vec![Expr::Field(field.clone())],
            Predicate::Regex { field, .. } => vec![Expr::Field(field.clone())],
            Predicate::In { field, .. } => {
                vec![Expr::Field(field.clone())]
            }
            Predicate::Between { field, .. } => vec![Expr::Field(field.clone())],
            // I-LOCAL-003: emit Compare so predicate_as_string can render the CIDR mask.
            // Using CompareOp::Cidr as the operator and Literal::Cidr as the rhs preserves
            // the network address for display (e.g. "src_ip CIDR '10.0.0.0/8'").
            //
            // I-LOCAL-NEW-1: honor the `negated` flag — a negated CIDR predicate
            // (`NOT src_ip CIDR '10.0.0.0/8'`) must render with a NOT prefix rather
            // than silently rendering identically to the non-negated form.
            // CompareOp::NotCidr is used so predicate_as_string can distinguish the two.
            Predicate::Cidr {
                field,
                cidr,
                negated,
            } => {
                let op = if *negated {
                    crate::ast::CompareOp::NotCidr
                } else {
                    crate::ast::CompareOp::Cidr
                };
                vec![Expr::Compare {
                    lhs: Box::new(Expr::Field(field.clone())),
                    op,
                    rhs: Box::new(Expr::Literal(crate::ast::Literal::Cidr(cidr.clone()))),
                }]
            }
            Predicate::Has(fp) => vec![Expr::Field(fp.clone())],
            Predicate::Missing(fp) => vec![Expr::Field(fp.clone())],
            Predicate::IsNull { field, .. } => vec![Expr::Field(field.clone())],
            Predicate::Wildcard { field, .. } => vec![Expr::Field(field.clone())],
            Predicate::InSubquery { .. } | Predicate::RecoveryError => vec![],
        }
    }

    match ast {
        Ast::Filter(fe) => predicate_to_exprs(&fe.predicate),
        Ast::Sql(SqlStatement::Select(sq)) => {
            let mut out = Vec::new();
            if let Some(w) = &sq.where_ {
                out.extend(predicate_to_exprs(w));
            }
            out
        }
        Ast::Pipe(pq) => {
            let mut out = Vec::new();
            for stage in &pq.stages {
                use crate::ast::PipeStage;
                if let PipeStage::Where(pred) = stage {
                    out.extend(predicate_to_exprs(pred));
                }
            }
            out
        }
        _ => vec![],
    }
}

// ---------------------------------------------------------------------------
// query_mode_str — mode label from AST
// ---------------------------------------------------------------------------

fn query_mode_str(ast: &Ast) -> &'static str {
    match ast {
        Ast::Filter(_) => "filter",
        Ast::Sql(_) => "sql",
        Ast::Pipe(_) => "pipe",
        // #[non_exhaustive] catch-all: future modes added by downstream stories.
        #[allow(unreachable_patterns)]
        _ => "unknown",
    }
}

// ---------------------------------------------------------------------------
// QueryEngine::explain (BC-2.11.010)
// ---------------------------------------------------------------------------

/// Cap the `query` field inserted into every `AuditEvent` to 256 chars.
///
/// SEC-P3-001: Both the size-guard path (E-QUERY-003) and the main `emit_audit`
/// closure must bound the logged query string so that a 65,535-byte alias
/// payload does not appear verbatim in audit logs.  256 chars is sufficient
/// for triage context while preventing unbounded log growth.
fn audit_query_field(s: &str) -> String {
    s.chars().take(256).collect()
}

/// Analyze a PrismQL query string and return an `ExplainResult` without
/// executing any sensor API calls.
///
/// # Contract (BC-2.11.010)
/// 1. Applies syntactic security limits (DI-019) via `PrismQlParser::parse`.
/// 2. Expands aliases from `options.alias_registry` before parsing.
/// 3. Resolves client scope via `scoping::resolve_clients` — NO fan-out.
/// 4. Classifies predicates via `pushdown::classify_predicates` — NO sensor calls.
/// 5. Builds cost estimate with per-sensor heuristics (read-only).
/// 6. Emits an audit event (DI-004) via `options.audit_sink` if provided.
///
/// # Returns
/// - `Ok(ExplainResult)` on success (including the EC-11-025 over-limit warning).
/// - `Err(PrismError)` for parse errors, alias errors, field errors, or security
///   limit violations.
///
/// # No sensor API calls
/// This function MUST NOT call `fan_out()`, any sensor adapter `fetch()`, or
/// any I/O path that reaches a sensor API endpoint. (BC-2.11.010 Postconditions)
pub fn explain(query_str: &str, options: ExplainOptions) -> Result<ExplainResult, PrismError> {
    // ── SEC-001: Pre-expansion size guard (defence-in-depth, E-QUERY-003) ─────
    // Checked before audit closure construction so the guard is the very first
    // operation. The parser also checks but library callers should not rely on
    // caller discipline.
    if query_str.len() > PRISM_MAX_QUERY_SIZE {
        // Emit audit for the raw oversized input before returning.
        if let Some(sink) = &options.audit_sink {
            sink(AuditEvent {
                query: audit_query_field(query_str),
                clients: options.clients.clone(),
                sensors: options.sensors.clone(),
                sources: options.sources.clone(),
                outcome_summary: "E-QUERY-003".to_string(),
            });
        }
        return Err(PrismError::QueryExecutionFailed {
            detail: format!(
                "E-QUERY-003: query size {} bytes exceeds maximum {} bytes",
                query_str.len(),
                PRISM_MAX_QUERY_SIZE
            ),
        });
    }

    // ── DI-004: helper to emit audit event (success AND error paths) ──────────
    // SEC-P3-001: cap query field in ALL audit emissions to 256 chars so that
    // the E-ALIAS-001 path (and any future error path) cannot leak unbounded
    // input. Matches the cap applied in the size-guard path above.
    let emit_audit = |outcome: &str| {
        if let Some(sink) = &options.audit_sink {
            sink(AuditEvent {
                query: audit_query_field(query_str),
                clients: options.clients.clone(),
                sensors: options.sensors.clone(),
                sources: options.sources.clone(),
                outcome_summary: outcome.to_string(),
            });
        }
    };

    // ── Step 1: Alias expansion (BC-2.11.010 postcondition `alias_expansion`) ─
    // Before parsing, attempt to expand the query if it's a bare alias reference.
    let (expanded_query, alias_expansion) =
        expand_query_with_aliases(query_str, &options.alias_registry).inspect_err(|_e| {
            emit_audit("E-ALIAS-001");
        })?;

    // ── Step 2: Security size check on expanded query (E-QUERY-003, DI-019) ───
    if expanded_query.len() > PRISM_MAX_QUERY_SIZE {
        emit_audit("E-QUERY-003");
        return Err(PrismError::QueryExecutionFailed {
            detail: format!(
                "E-QUERY-003: expanded query size {} bytes exceeds maximum allowed {} bytes",
                expanded_query.len(),
                PRISM_MAX_QUERY_SIZE
            ),
        });
    }

    // ── Step 3: Parse (enforces all security limits: size, depth, pipe stages) ─
    let ast = PrismQlParser::parse(&expanded_query).map_err(|errs| {
        let detail = errs
            .first()
            .map(|e| e.to_string())
            .unwrap_or_else(|| "parse failed".to_string());
        emit_audit("E-QUERY-001");
        PrismError::QueryParseFailed { offset: 0, detail }
    })?;

    let parsed_mode = query_mode_str(&ast).to_string();

    // ── Step 4: Extract sources from AST ──────────────────────────────────────
    let mut raw_sources = extract_sources_from_ast(&ast);

    // Apply sensor scope filter from options.
    // I-LOCAL-002 (BC-2.11.010 v1.4 Preconditions): the sensors filter applies only to
    // external sensor sources (SourceRefKind::External). Non-external sources (internal,
    // composite) are sensor-agnostic and are dropped when a sensor scope filter is active —
    // they cannot be validated against any specific sensor type. This is intentional.
    if let Some(sensor_scope) = &options.sensors {
        raw_sources.retain(|s| {
            if let Some(st) = sensor_type_from_source_ref(s) {
                sensor_scope.contains(&st)
            } else {
                false
            }
        });
    }

    // Apply sources scope filter from options.
    if let Some(sources_scope) = &options.sources {
        raw_sources.retain(|s| sources_scope.iter().any(|sc| sc == &s.raw));
    }

    // ── Step 5: Extract predicates for push-down classification ──────────────
    let where_exprs = predicates_from_ast(&ast);

    // ── Step 6: Build ExplainSource entries per sensor source ─────────────────
    // No ColumnSpec is available without a sensor schema registry; we use an
    // empty spec list, which means all predicates fall to post-filter (conservative).
    // When sensor schema specs are wired (S-3.X), pass the real ColumnSpec list here.
    //
    // `classify_predicates` result is invariant across sources at this stage (no
    // per-sensor ColumnSpec available), so hoist outside the filter_map to avoid
    // redundant recomputation on each iteration.
    // TODO(S-3.X): per-sensor ColumnSpec — replace the empty `&[]` and move back
    // inside the loop once the sensor schema registry is wired.
    let plan = classify_predicates(&where_exprs, &[]);
    let sensors_to_query: Vec<ExplainSource> = raw_sources
        .iter()
        .filter_map(|s| {
            let sensor_type = sensor_type_from_source_ref(s)?;
            // `plan` is shared across all sources at this stage (no per-sensor ColumnSpec).

            let api_filters: Vec<String> = plan
                .push_down
                .iter()
                .map(|p| predicate_as_string(&p.expr, &p.column_name))
                .collect();

            let post_filter: Vec<String> = plan
                .post_filter
                .iter()
                .map(|p| predicate_as_string(&p.expr, &p.column_name))
                .collect();

            Some(ExplainSource {
                source_ref: s.raw.clone(),
                sensor_type,
                api_filters_pushed: api_filters,
                post_filter_predicates: post_filter,
                estimated_row_count: None, // S-3.X: wire count_hint() here
            })
        })
        .collect();

    // ── Step 7: Resolve client scope (no fan-out) ─────────────────────────────
    // AC-5: resolve_clients returns the list of all configured client IDs when
    // options.clients is None (no fan-out; pure registry lookup). The resolved
    // list is recorded in ExecutionPlan.clients_to_query for caller visibility.
    let empty_registry = ClientRegistry::default();
    let client_registry = options
        .client_registry
        .as_deref()
        .unwrap_or(&empty_registry);
    let resolved_clients =
        resolve_clients(options.clients.clone(), client_registry).inspect_err(|_e| {
            emit_audit("E-MCP-004");
        })?;

    // ── Step 8: Post-fetch operations from AST ────────────────────────────────
    let post_fetch_operations = post_fetch_operations_from_ast(&ast);

    // ── Step 9: Field resolution (BC-2.11.010 postcondition `field_resolution`) ─
    let mut field_collector = FieldCollector::new();
    walk_ast(&mut field_collector, &ast);

    let mut field_resolution: HashMap<String, FieldResolution> = HashMap::new();

    // Virtual fields resolve as "virtual".
    for vf_name in &field_collector.virtual_fields {
        field_resolution.insert(
            vf_name.clone(),
            FieldResolution {
                ocsf_path: ocsf_path_for_virtual_field(vf_name),
                resolution_method: "virtual".to_string(),
            },
        );
    }

    // Regular fields resolve as "direct".
    //
    // TODO(S-3.X / TD-S303-ALIAS-PROVENANCE-001): When an alias is expanded,
    // track which AST fields originated from the expansion text. Currently
    // all non-virtual fields are labeled "direct" regardless of alias provenance.
    // Implementing alias-provenance tracking requires the parser to retain
    // origin metadata or a post-expansion lightweight field-extraction step.
    for field_name in &field_collector.fields {
        field_resolution.insert(
            field_name.clone(),
            FieldResolution {
                ocsf_path: ocsf_path_for_field(field_name),
                resolution_method: "direct".to_string(),
            },
        );
    }

    // ── Step 10: Build CostEstimate ───────────────────────────────────────────
    let mut warnings: Vec<String> = Vec::new();

    // I-LOCAL-004: warn when WHERE clause contains OR predicates.
    // Flattening OR branches into separate api_filters_pushed entries is misleading
    // because OR semantics require ALL branches to match OR (not AND). The current
    // push-down classifier treats each flattened clause independently and conservatively
    // assigns them to post_filter; however, the caller sees a warning to prompt
    // awareness. S-3.X will implement OR-mode handling with sensor-native OR translation.
    if has_or_predicate(&ast) {
        warnings.push(
            "OR predicate detected — push-down semantics may differ from execution; \
             conservatively reporting all OR clauses as post-filter. \
             S-3.X will refine OR-mode handling."
                .to_string(),
        );
    }

    // Per-sensor cost heuristics (static estimates; S-3.X wires real telemetry).
    let mut per_sensor_latency_ms: HashMap<String, u64> = HashMap::new();
    let mut per_sensor_api_call_count: HashMap<String, u64> = HashMap::new();
    let mut per_sensor_rate_limit_headroom: HashMap<String, u64> = HashMap::new();

    for src in &sensors_to_query {
        let sensor_key = src.sensor_type.to_string();

        // Heuristic: base latency by sensor id (will be replaced by real metrics).
        // Open dispatch: unknown sensors fall to the default case.
        let latency_ms = match src.sensor_type.as_ref() {
            "crowdstrike" => 250,
            "cyberint" => 400,
            "claroty" => 350,
            "armis" => 300,
            _ => 300,
        };
        per_sensor_latency_ms.insert(sensor_key.clone(), latency_ms);

        // Heuristic: API call count (at least 1; more with pagination).
        per_sensor_api_call_count.insert(sensor_key.clone(), 1);

        // Heuristic: rate limit headroom (unknown without real telemetry → 100).
        per_sensor_rate_limit_headroom.insert(sensor_key.clone(), 100);

        // EC-11-025: warn if estimated_row_count > 10K.
        if let Some(count) = src.estimated_row_count {
            if count > 10_000 {
                warnings.push(format!(
                    "sensor '{}' source '{}' has estimated row count {} > 10,000; \
                     the query would fail at execution time due to the materialization limit (DI-019)",
                    sensor_key, src.source_ref, count
                ));
            }
        }
    }

    let summary = if sensors_to_query.is_empty() {
        "No sensor sources identified. Query may target internal or composite tables.".to_string()
    } else {
        let total_latency: u64 = per_sensor_latency_ms.values().copied().max().unwrap_or(0);
        let total_calls: u64 = per_sensor_api_call_count.values().copied().sum();
        format!(
            "Estimated max sensor latency: {}ms. Total estimated API calls: {}. \
             Sensors: {}",
            total_latency,
            total_calls,
            sensors_to_query
                .iter()
                .map(|s| s.sensor_type.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    };

    // ── Step 11: Assemble result ───────────────────────────────────────────────
    let result = ExplainResult {
        parsed_mode,
        original_query: query_str.to_string(),
        alias_expansion: alias_expansion
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
        expanded_query,
        field_resolution,
        execution_plan: ExecutionPlan {
            sensors_to_query,
            post_fetch_operations,
            clients_to_query: resolved_clients,
        },
        estimated_cost: CostEstimate {
            per_sensor_latency_ms,
            per_sensor_api_call_count,
            per_sensor_rate_limit_headroom,
            summary,
            warnings,
        },
    };

    // ── DI-004: Emit success audit event ──────────────────────────────────────
    emit_audit("success");

    Ok(result)
}

// ---------------------------------------------------------------------------
// expand_query_with_aliases — alias registry expansion
// ---------------------------------------------------------------------------

/// Expand a query string using the alias registry.
///
/// Returns `(expanded_query_string, alias_map_used)` where `alias_map_used`
/// contains only the aliases that were actually applied (may be empty).
///
/// # Expansion semantics (SEC-P2-003)
///
/// Alias expansion is **single-pass**: the registry is iterated once, sorted
/// by alias name length descending (longest-match-first) so that a longer
/// alias always wins over a shorter prefix alias (e.g. `"sev_critical"` beats
/// `"sev"` when both match). After the first matching alias is applied, no
/// further substitution is attempted on the result. This means aliases that
/// happen to begin with another alias's expansion text are **not** recursively
/// re-expanded.
///
/// This is intentional: aliases are currently system-defined (not
/// user-writable), so recursive re-expansion would add complexity with no
/// practical benefit. If alias storage ever becomes user-controlled, recursive
/// expansion would be a potential injection vector and must be revisited at
/// that time.
///
/// # Error cases
/// - E-ALIAS-001: query references an alias name that is not in the registry
///   AND the caller has explicitly flagged it as an alias reference via the
///   `@alias:` prefix convention.
fn expand_query_with_aliases(
    query_str: &str,
    alias_registry: &HashMap<String, String>,
) -> Result<(String, HashMap<String, String>), PrismError> {
    // Check for explicit alias reference syntax: `@alias:<name>` prefix.
    // This is distinct from bare-identifier detection because it is unambiguous.
    if let Some(alias_name) = query_str.strip_prefix("@alias:") {
        let alias_name = alias_name.trim();
        match alias_registry.get(alias_name) {
            Some(expansion) => {
                let mut used = HashMap::new();
                used.insert(alias_name.to_string(), expansion.clone());
                return Ok((expansion.clone(), used));
            }
            None => {
                // SEC-002 / CR-017: cap alias name echo to prevent unbounded string in
                // error message. CR-019: guard on char count (not byte len) so that
                // multi-byte UTF-8 aliases ≤ 64 chars are not falsely truncated with
                // a misleading "…".
                let alias_display = if alias_name.chars().count() > 64 {
                    let truncated: String = alias_name.chars().take(64).collect();
                    format!("{truncated}...")
                } else {
                    alias_name.to_string()
                };
                return Err(PrismError::QueryExecutionFailed {
                    detail: format!(
                        "E-ALIAS-001: alias '{alias_display}' is not defined in the alias registry"
                    ),
                });
            }
        }
    }

    // Standard query: perform token-level alias substitution.
    // For each alias in the registry, if the query begins with the alias
    // name (as a complete token), substitute it.
    //
    // CR-005: Sort by alias name length descending so longer (more specific)
    // aliases win over shorter ones when prefixes overlap (e.g. "ab" beats "a"
    // for input "ab <rest>"). This eliminates HashMap iteration non-determinism.
    let trimmed = query_str.trim();
    let mut used: HashMap<String, String> = HashMap::new();
    let mut expanded = trimmed.to_string();

    let mut pairs: Vec<(&String, &String)> = alias_registry.iter().collect();
    pairs.sort_by_key(|(k, _)| std::cmp::Reverse(k.len()));

    for (alias_name, expansion) in pairs {
        // Match alias at word boundaries.
        if expanded == alias_name.as_str()
            || expanded.starts_with(&format!("{alias_name} "))
            || expanded.starts_with(&format!("{alias_name}\t"))
            || expanded.starts_with(&format!("{alias_name}|"))
        {
            expanded = expanded.replacen(alias_name.as_str(), expansion.as_str(), 1);
            used.insert(alias_name.clone(), expansion.clone());
            break; // first-match wins per SEC-P2-003 single-pass invariant
        }
    }

    Ok((expanded, used))
}

// ---------------------------------------------------------------------------
// predicate_as_string — human-readable predicate representation
// ---------------------------------------------------------------------------

/// Format a predicate expression as a human-readable string.
///
/// Used to populate `api_filters_pushed` and `post_filter_predicates`.
/// The output is PrismQL-native (informational). Sensor-specific translation
/// (FQL, KQL, etc.) is deferred to S-3.X — see TD-S303-PUSH-DOWN-TRANSLATION-001.
// CR-002: translate to sensor-native syntax (TD-S303-PUSH-DOWN-TRANSLATION-001)
fn predicate_as_string(expr: &crate::ast::Expr, column_name: &str) -> String {
    use crate::ast::{CompareOp, Expr, Literal};

    match expr {
        Expr::Compare { lhs: _, op, rhs } => {
            // I-LOCAL-NEW-1: NotCidr is a negated CIDR predicate — render with NOT prefix.
            let is_not_cidr = matches!(op, CompareOp::NotCidr);
            let op_str = match op {
                CompareOp::Eq => "=",
                CompareOp::Ne => "!=",
                CompareOp::Lt => "<",
                CompareOp::Le => "<=",
                CompareOp::Gt => ">",
                CompareOp::Ge => ">=",
                CompareOp::Like => "LIKE",
                CompareOp::Cidr => "CIDR",
                // I-LOCAL-NEW-1: negated CIDR renders the same operator token; the NOT
                // prefix is prepended to the whole expression below.
                CompareOp::NotCidr => "CIDR",
                // #[non_exhaustive] catch-all for future operator variants.
                #[allow(unreachable_patterns)]
                _ => "?",
            };
            let rhs_str = match rhs.as_ref() {
                Expr::Literal(lit) => match lit {
                    Literal::String(s) => format!("'{s}'"),
                    Literal::Integer(n) => n.to_string(),
                    Literal::Float(f) => f.to_string(),
                    Literal::Bool(b) => b.to_string(),
                    // I-LOCAL-003: render CIDR mask so predicate is not silently truncated.
                    Literal::Cidr(c) => format!("'{}'", c.cidr),
                    #[allow(unreachable_patterns)]
                    _ => "<literal>".to_string(),
                },
                _ => "<expr>".to_string(),
            };
            let base = format!("{column_name} {op_str} {rhs_str}");
            if is_not_cidr {
                format!("NOT {base}")
            } else {
                base
            }
        }
        _ => column_name.to_string(),
    }
}

// ---------------------------------------------------------------------------
// ocsf_path_for_field — heuristic OCSF path mapping
// ---------------------------------------------------------------------------

/// Return a heuristic OCSF path for a field name.
///
/// This is a best-effort mapping for explain output. Full schema-driven
/// resolution requires the sensor spec engine (S-3.X). Common fields are
/// mapped to their canonical OCSF paths; unknown fields use a generic path.
fn ocsf_path_for_field(field_name: &str) -> String {
    // Common OCSF field mappings used in tests and documentation.
    match field_name {
        "severity" | "severity_id" => "finding.severity_id".to_string(),
        "hostname" | "device_name" => "device.hostname".to_string(),
        "status" | "status_id" => "finding.status_id".to_string(),
        "alert_id" | "detection_id" => "finding.uid".to_string(),
        "timestamp" | "created_time" => "metadata.original_time".to_string(),
        "src_ip" | "source_ip" => "src_endpoint.ip".to_string(),
        "dst_ip" | "dest_ip" | "destination_ip" => "dst_endpoint.ip".to_string(),
        "type" | "type_id" => "finding.type_id".to_string(),
        "message" | "description" => "finding.message".to_string(),
        "count" => "finding.count".to_string(),
        _ => format!("finding.{field_name}"),
    }
}

/// Return the OCSF path for a virtual field.
fn ocsf_path_for_virtual_field(name: &str) -> String {
    match name {
        "_sensor" => "metadata.sensor_type".to_string(),
        "_client" => "metadata.org_id".to_string(),
        "_source_table" => "metadata.source_table".to_string(),
        "_source_type" => "metadata.source_type".to_string(),
        "_safety_flags" => "metadata.safety_flags".to_string(),
        _ => format!("metadata.{name}"),
    }
}

// ---------------------------------------------------------------------------
// Tests — extract_sources_from_ast walker coverage
// ---------------------------------------------------------------------------

#[cfg(test)]
mod walker_coverage_tests {
    //! Layer 1 AST walker coverage tests for `extract_sources_from_ast`.
    //!
    //! These tests build AST structures directly (no parser) and assert that
    //! `extract_sources_from_ast` discovers every source table name, including
    //! those carried by DML nodes (F-LP6-LOW-1).

    use super::extract_sources_from_ast;
    use crate::ast::{
        Ast, Expr, FieldPath, FromClause, SelectClause, SelectItem, SourceRef, SourceRefKind, Span,
        SqlQuery, SqlStatement,
    };

    // Helper: build a minimal SourceRef with raw table name.
    fn source_ref(name: &str) -> SourceRef {
        SourceRef {
            raw: name.to_string(),
            kind: SourceRefKind::Custom,
        }
    }

    // Helper: build a minimal FromClause.
    fn from_clause(name: &str) -> FromClause {
        FromClause {
            source: source_ref(name),
            alias: None,
        }
    }

    /// F-LP6-LOW-1: DML source_select sources must appear in explain sensors_to_query.
    ///
    /// Represents: `INSERT INTO crowdstrike_contained_hosts SELECT host_id FROM prism_audit`
    ///
    /// EXPLAIN output's "sensors_to_query" must reflect DML source_select sources so
    /// that operators see the full query plan for DML queries.
    /// Lineage: F-LP3-CRIT-1 → F-LP4-MED-1 → F-LP5-LOW-1 → F-LP6-LOW-1.
    #[test]
    #[allow(non_snake_case)]
    fn test_LP6_LOW_1_dml_source_select_appears_in_explain_sensors() {
        use crate::write_ast::{DmlNode, DmlOperation};

        // Build: INSERT INTO crowdstrike_contained_hosts SELECT host_id FROM prism_audit
        let source_select = SqlQuery {
            select: SelectClause {
                distinct: false,
                items: vec![SelectItem::Expr {
                    expr: Expr::Field(FieldPath {
                        segments: vec!["host_id".to_string()],
                        span: Span::ZERO,
                    }),
                    alias: None,
                }],
            },
            from: from_clause("prism_audit"),
            joins: vec![],
            where_: None,
            group_by: vec![],
            having: None,
            order_by: vec![],
            limit: None,
        };

        let dml = DmlNode {
            operation: DmlOperation::InsertInto,
            target_table: "crowdstrike_contained_hosts".to_string(),
            columns: None,
            assignments: vec![],
            filter: None,
            source_select: Some(source_select),
        };

        let ast = Ast::Sql(SqlStatement::Dml(dml));
        let sources = extract_sources_from_ast(&ast);

        assert!(
            sources.iter().any(|s| s.raw == "prism_audit"),
            "F-LP6-LOW-1: extract_sources_from_ast must discover `prism_audit` \
             in DML source_select (INSERT INTO ... SELECT FROM prism_audit); got sources: {sources:?}"
        );
    }
}
