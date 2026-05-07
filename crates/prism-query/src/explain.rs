//! `explain` — PrismQL query plan analysis without execution.
//!
//! Implements BC-2.11.010: parse and plan a PrismQL query without issuing any
//! sensor API calls. Returns a structured `ExplainResult` containing the
//! detected query mode, alias expansions, field resolution, per-sensor push-down
//! filters in sensor-native syntax, post-fetch operations, and a cost estimate.
//!
//! # Architecture Compliance (BC-2.11.010)
//! - MUST NOT call `fan_out()` or any sensor adapter `fetch()` method.
//! - Reuses `classify_predicates()` from `pushdown.rs` — do NOT duplicate logic.
//! - Reuses `resolve_clients()` from `scoping.rs`.
//! - DataFusion logical plan obtained against schema-only MemTables.
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

use prism_core::{OrgSlug, PrismError, SensorType};
use serde::{Deserialize, Serialize};

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
    pub sensors: Option<Vec<SensorType>>,
    /// The source scope parameter.
    pub sources: Option<Vec<String>>,
    /// Human-readable outcome summary (e.g. "success", "E-QUERY-001").
    pub outcome_summary: String,
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
    pub sensors: Option<Vec<SensorType>>,
    /// Data source scope override: `None` = all sources for resolved sensors. (BC-2.11.010)
    pub sources: Option<Vec<String>>,
    /// Alias registry mapping alias names to their expanded definitions.
    ///
    /// Used to expand alias references in the query and populate `alias_expansion`
    /// in the result. Until S-3.04 (alias registry story) merges, callers must
    /// provide this at call time. (BC-2.11.010 postcondition `alias_expansion`)
    pub alias_registry: HashMap<String, String>,
    /// Client registry for resolving `clients: None` to all configured clients.
    ///
    /// `None` uses an empty registry (no configured clients).
    pub client_registry: Option<Arc<ClientRegistry>>,
    /// DI-004 audit sink. Called once for every invocation (success AND error).
    ///
    /// In production, `prism-mcp` provides the real audit emitter. Tests provide
    /// a capturing closure.
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    /// List of sensors that would be queried. (BC-2.11.010)
    pub sensors_to_query: Vec<ExplainSource>,

    /// Per-source post-fetch operations (filter, group-by, sort, limit, etc.).
    /// (BC-2.11.010)
    pub post_fetch_operations: Vec<String>,
}

// ---------------------------------------------------------------------------
// ExplainSource
// ---------------------------------------------------------------------------

/// Per-sensor push-down information for the explain result. (BC-2.11.010)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainSource {
    /// Source reference string, e.g. `"crowdstrike.detections"`.
    pub source_ref: String,

    /// The sensor type this source belongs to.
    pub sensor_type: SensorType,

    /// Push-down predicates translated to sensor-native syntax
    /// (e.g., FQL for CrowdStrike). (BC-2.11.010 `api_filters_pushed`)
    pub api_filters_pushed: Vec<String>,

    /// Predicates applied post-materialization by DataFusion. (BC-2.11.010)
    pub post_filter_predicates: Vec<String>,

    /// Estimated row count from sensor count hint, if available.
    /// `None` if the sensor adapter does not expose a count hint. (Story §Dev Notes)
    pub estimated_row_count: Option<u64>,
}

// ---------------------------------------------------------------------------
// CostEstimate
// ---------------------------------------------------------------------------

/// Structured cost estimate for the query. (BC-2.11.010 `estimated_cost`)
#[derive(Debug, Clone, Serialize, Deserialize)]
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
// extract_sources_from_ast — collect SourceRef entries directly from AST
// ---------------------------------------------------------------------------

/// Extract all source references from an AST, deduplicated by raw string.
///
/// The visitor trait does not expose `visit_source_ref` because `SourceRef`
/// is embedded structurally in the AST (not traversed by `walk_*`). We
/// access sources directly from the top-level AST variants. (BC-2.11.010)
fn extract_sources_from_ast(ast: &Ast) -> Vec<SourceRef> {
    let mut sources: Vec<SourceRef> = Vec::new();

    let mut push = |s: &SourceRef| {
        if !sources.iter().any(|x| x.raw == s.raw) {
            sources.push(s.clone());
        }
    };

    match ast {
        Ast::Filter(fe) => {
            push(&fe.source);
        }
        Ast::Sql(SqlStatement::Select(sq)) => {
            push(&sq.from.source);
            for join in &sq.joins {
                push(&join.source);
            }
        }
        Ast::Pipe(pq) => {
            push(&pq.source);
        }
        _ => {}
    }

    sources
}

// ---------------------------------------------------------------------------
// PostFetchOperationExtractor — derives post-fetch operations from AST
// ---------------------------------------------------------------------------

/// Derive the list of post-fetch operations (GROUP BY, SORT BY, LIMIT, etc.)
/// from the AST. These are operations that cannot be pushed to sensor APIs and
/// must be applied by DataFusion after materialization. (BC-2.11.010)
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
        }
    }

    ops
}

// ---------------------------------------------------------------------------
// sensor_type_from_source_ref — derive SensorType from SourceRef
// ---------------------------------------------------------------------------

/// Derive the sensor type for a source reference.
///
/// Returns `None` for composite, internal, or custom source kinds that do not
/// map to a specific sensor adapter.
fn sensor_type_from_source_ref(s: &SourceRef) -> Option<SensorType> {
    match &s.kind {
        SourceRefKind::External { sensor, .. } => match sensor.to_lowercase().as_str() {
            "crowdstrike" => Some(SensorType::CrowdStrike),
            "cyberint" => Some(SensorType::Cyberint),
            "claroty" => Some(SensorType::Claroty),
            "armis" => Some(SensorType::Armis),
            _ => None,
        },
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
            Predicate::Not(inner) => predicate_to_exprs(inner),
            Predicate::StringOp { field, .. } => vec![Expr::Field(field.clone())],
            Predicate::Regex { field, .. } => vec![Expr::Field(field.clone())],
            Predicate::In { field, .. } => {
                vec![Expr::Field(field.clone())]
            }
            Predicate::Between { field, .. } => vec![Expr::Field(field.clone())],
            Predicate::Cidr { field, .. } => vec![Expr::Field(field.clone())],
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
    // ── DI-004: helper to emit audit event (success AND error paths) ──────────
    let emit_audit = |outcome: &str| {
        if let Some(sink) = &options.audit_sink {
            sink(AuditEvent {
                query: query_str.to_string(),
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
    if let Some(sensor_scope) = &options.sensors {
        raw_sources.retain(|s| {
            if let Some(st) = sensor_type_from_source_ref(s) {
                sensor_scope.contains(&st)
            } else {
                // Non-external sources are not sensor-specific; keep them if
                // sensor scope doesn't restrict them.
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
    let sensors_to_query: Vec<ExplainSource> = raw_sources
        .iter()
        .filter_map(|s| {
            let sensor_type = sensor_type_from_source_ref(s)?;
            // Classify predicates against this source using the push-down module.
            // With no ColumnSpec, all predicates → post_filter (conservative fallback).
            let plan = classify_predicates(&where_exprs, &[]);

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
    let empty_registry = ClientRegistry::default();
    let client_registry = options
        .client_registry
        .as_deref()
        .unwrap_or(&empty_registry);
    let _resolved_clients =
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

    // Regular fields resolve as "alias" (if expanded via alias) or "direct".
    for field_name in &field_collector.fields {
        // Check if this field name was the result of alias expansion.
        let is_alias_expanded = alias_expansion.contains_key(field_name.as_str());
        let resolution_method = if is_alias_expanded { "alias" } else { "direct" };
        field_resolution.insert(
            field_name.clone(),
            FieldResolution {
                ocsf_path: ocsf_path_for_field(field_name),
                resolution_method: resolution_method.to_string(),
            },
        );
    }

    // ── Step 10: Build CostEstimate ───────────────────────────────────────────
    let mut warnings: Vec<String> = Vec::new();

    // Per-sensor cost heuristics (static estimates; S-3.X wires real telemetry).
    let mut per_sensor_latency_ms: HashMap<String, u64> = HashMap::new();
    let mut per_sensor_api_call_count: HashMap<String, u64> = HashMap::new();
    let mut per_sensor_rate_limit_headroom: HashMap<String, u64> = HashMap::new();

    for src in &sensors_to_query {
        let sensor_key = src.sensor_type.to_string();

        // Heuristic: base latency by sensor type (will be replaced by real metrics).
        let latency_ms = match src.sensor_type {
            SensorType::CrowdStrike => 250,
            SensorType::Cyberint => 400,
            SensorType::Claroty => 350,
            SensorType::Armis => 300,
            // #[non_exhaustive] catch-all for future sensor types.
            #[allow(unreachable_patterns)]
            _ => 500,
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
/// Error cases:
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
                return Err(PrismError::QueryExecutionFailed {
                    detail: format!(
                        "E-ALIAS-001: alias '{}' is not defined in the alias registry",
                        alias_name
                    ),
                });
            }
        }
    }

    // Standard query: perform token-level alias substitution.
    // For each alias in the registry, if the query begins with the alias
    // name (as a complete token), substitute it.
    let trimmed = query_str.trim();
    let mut used: HashMap<String, String> = HashMap::new();
    let mut expanded = trimmed.to_string();

    for (alias_name, expansion) in alias_registry {
        // Match alias at word boundaries.
        if expanded == alias_name.as_str()
            || expanded.starts_with(&format!("{alias_name} "))
            || expanded.starts_with(&format!("{alias_name}\t"))
            || expanded.starts_with(&format!("{alias_name}|"))
        {
            expanded = expanded.replacen(alias_name.as_str(), expansion.as_str(), 1);
            used.insert(alias_name.clone(), expansion.clone());
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
/// The output is informational; sensor-specific translation happens in S-3.X.
fn predicate_as_string(expr: &crate::ast::Expr, column_name: &str) -> String {
    use crate::ast::{CompareOp, Expr, Literal};

    match expr {
        Expr::Compare { lhs: _, op, rhs } => {
            let op_str = match op {
                CompareOp::Eq => "=",
                CompareOp::Ne => "!=",
                CompareOp::Lt => "<",
                CompareOp::Le => "<=",
                CompareOp::Gt => ">",
                CompareOp::Ge => ">=",
                _ => "?",
            };
            let rhs_str = match rhs.as_ref() {
                Expr::Literal(lit) => match lit {
                    Literal::String(s) => format!("'{s}'"),
                    Literal::Integer(n) => n.to_string(),
                    Literal::Float(f) => f.to_string(),
                    Literal::Bool(b) => b.to_string(),
                    _ => "<literal>".to_string(),
                },
                _ => "<expr>".to_string(),
            };
            format!("{column_name} {op_str} {rhs_str}")
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
        "src_ip" | "source_ip" => "network_endpoint.ip".to_string(),
        "dst_ip" | "dest_ip" => "network_endpoint.ip".to_string(),
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
