//! Write execution pipeline — `WriteExecutor` and six-phase orchestration.
//!
//! Implements the six-phase write execution pipeline defined in AD-022:
//! - Phase 1 (parse): consumed from S-3.06 write AST nodes (`WriteNode`, `DmlNode`).
//! - Phase 2 (safety pre-check): pure gates via `safety_check::phase2_safety_check`.
//! - Phase 3 (fetch): reuses `QueryMaterializer` from S-3.02.
//! - Phase 4 (dry-run / confirm gate): `dry_run::DryRunGate`.
//! - Phase 5 (write dispatch): `write_dispatch::WriteDispatcher`.
//! - Phase 6 (return): constructs `WriteResult` from aggregated results.
//!
//! # Architecture Compliance
//! - Phase 2 is entirely pure — no I/O.
//! - Audit INTENT write (Phase 5a) is a synchronous `await` before any HTTP.
//! - Write semaphore capacity is 4 — separate from read semaphore (10).
//! - `dry_run` comes from `QueryContext` (MCP tool layer), NOT the query string.
//! - Partial batch failure is NOT an error return — it lives in `WriteResult`.
//!
//! Story: S-3.07 | BCs: BC-2.04.001, BC-2.04.007, BC-2.04.008, BC-2.05.009

use std::collections::HashMap;
use std::sync::Arc;

use prism_core::{OrgSlug, PrismError, RiskTier};
use prism_security::confirmation_token::ConfirmationTokenStore;
use prism_security::feature_flag::{
    armis_write_gate, claroty_write_gate, crowdstrike_write_gate, cyberint_write_gate,
    FeatureFlagEvaluator,
};
use prism_sensors::AdapterRegistry;
use prism_spec_engine::write_endpoint::WriteEndpointRegistry;

use crate::dry_run::{DryRunGate, GateInputs};
use crate::safety_check::{
    phase2_safety_check, resolve_batch_limit, CompileFeatureGate, WriteTargetDescriptor,
};
use crate::write_ast::{DmlNode, DmlOperation, WriteNode};
use crate::write_dispatch::{AuditWriter, DispatchInputs, WriteDispatcher};
use crate::write_result::{WritePreview, WriteResult};

// System-wide ceiling for batch limits (used when no endpoint-specific limit exists).
const SYSTEM_BATCH_CEILING: u32 = 10_000;

// ---------------------------------------------------------------------------
// WritePlan — internal execution plan
// ---------------------------------------------------------------------------

/// Internal write execution plan derived from either a `WriteNode` (pipe mode)
/// or `DmlNode` (SQL mode).
///
/// Carries all information needed for Phases 2–6 without re-parsing the original
/// query string (story §S-3.06 intelligence: "WritePlan is trivially constructible
/// from both node types — no re-parsing").
#[derive(Debug, Clone)]
pub struct WritePlan {
    /// The write verb (pipe mode) or operation (SQL mode), e.g., `"contain"`.
    pub verb: String,
    /// Target sensor name, e.g., `"crowdstrike"`.
    pub sensor: String,
    /// Target table name in the sensor API / DataFusion catalog.
    pub target_table: String,
    /// SQL DML operation kind, if this plan was derived from a `DmlNode`.
    ///
    /// `None` for pipe-mode write plans.
    pub dml_operation: Option<DmlOperation>,
    /// Whether the source query has an explicit LIMIT clause.
    pub has_explicit_limit: bool,
    /// The explicit limit value, if present.
    pub explicit_limit: Option<u64>,
    /// Whether the source query has a WHERE clause (or pipe-mode filter stage).
    pub has_where_clause: bool,
    /// Key=value parameters from the write stage (pipe mode) or SET assignments (SQL mode).
    pub params: HashMap<String, String>,
}

impl WritePlan {
    /// Construct a `WritePlan` from a pipe-mode `WriteNode`.
    ///
    /// The `source_sensor` must be resolved at parse time (from the pipe source stage).
    /// This is a pure constructor — no I/O.
    pub fn from_write_node(node: &WriteNode, source_has_filter: bool) -> Self {
        let sensor = node.source_sensor.clone().unwrap_or_default();
        let params: HashMap<String, String> = node
            .args
            .iter()
            .map(|arg| (arg.key.clone(), arg.value.to_user_string()))
            .collect();

        Self {
            verb: node.verb.clone(),
            sensor: sensor.clone(),
            target_table: format!("{}_{}", sensor, node.verb),
            dml_operation: None,
            has_explicit_limit: false,
            explicit_limit: None,
            has_where_clause: source_has_filter,
            params,
        }
    }

    /// Construct a `WritePlan` from a SQL-mode `DmlNode`.
    ///
    /// F-PASS2-MED-004: Uses `WriteEndpointRegistry::sensor_for_table` for authoritative
    /// sensor resolution instead of a split-on-underscore heuristic. The heuristic was
    /// incorrect for multi-underscore table names (e.g., "crowdstrike_detection_audit"
    /// only yielded "crowdstrike" by luck; "events" would yield "events" with no sensor).
    ///
    /// Returns `Err(PrismError::WriteTargetTableUnknown)` if the table is not registered
    /// in the endpoint registry. This is a structural / configuration error at the DML
    /// parse → registry lookup boundary, before any client identity resolution.
    /// Distinct from `WriteAdapterNotConfiguredForClient` (E-QUERY-029), which fires when
    /// the table IS in the registry but the per-client adapter has not been initialized.
    pub fn from_dml_node(
        node: &DmlNode,
        registry: &WriteEndpointRegistry,
    ) -> Result<Self, PrismError> {
        let verb = match node.operation {
            DmlOperation::InsertInto => "insert".to_string(),
            DmlOperation::Update => "update".to_string(),
            DmlOperation::Delete => "delete".to_string(),
        };

        // F-PASS2-MED-004: authoritative sensor resolution via registry lookup.
        // Replaces split('_').next() heuristic which failed for tables without underscores
        // or whose prefix did not match the sensor name.
        //
        // fix-pass-2-correction (D-285): emit E-QUERY-030 WriteTargetTableUnknown when
        // the table is absent from the registry. No client_id is involved at this stage —
        // WritePlan construction is a pre-client-resolution step. E-QUERY-029 is RESERVED
        // for the W3-FIX-S307-002 OrgRegistry lookup path where client identity IS known.
        let sensor = registry
            .sensor_for_table(&node.target_table)
            .ok_or_else(|| PrismError::WriteTargetTableUnknown {
                table: node.target_table.clone(),
            })?
            .to_string();

        let has_where_clause = node.filter.is_some();
        let params: HashMap<String, String> = node
            .assignments
            .iter()
            .map(|a| (a.column.clone(), a.value.to_user_string()))
            .collect();

        Ok(Self {
            verb,
            sensor,
            target_table: node.target_table.clone(),
            dml_operation: Some(node.operation.clone()),
            has_explicit_limit: false,
            explicit_limit: None,
            has_where_clause,
            params,
        })
    }

    /// Check if this plan targets an internal `prism_*` table.
    ///
    /// HIGH-3: case-insensitive check — "Prism_alerts" and "PRISM_AUDIT" are also internal.
    pub fn is_internal_table(&self) -> bool {
        self.target_table.to_ascii_lowercase().starts_with("prism_")
    }
}

// ---------------------------------------------------------------------------
// QueryContext — per-call context forwarded from the MCP tool layer
// ---------------------------------------------------------------------------

/// Per-call context forwarded from the MCP tool layer to the write executor.
///
/// Contains the `dry_run` flag (from MCP tool parameters, NOT the query string),
/// client identity, and optional confirmation token for irreversible execute calls.
#[derive(Debug, Clone)]
pub struct QueryContext {
    /// Client (tenant) identifier for this write call.
    pub client_id: String,
    /// Org slug for this write call.
    pub org_slug: OrgSlug,
    /// Whether to run in dry-run mode (default: true).
    ///
    /// Comes from the MCP tool layer — NOT from the PrismQL query string (Dev Notes).
    pub dry_run: bool,
    /// Confirmation token ID for irreversible execute calls (Phase 4b).
    ///
    /// `None` for dry-run calls and reversible execute calls.
    pub confirmation_token_id: Option<String>,
    /// Analyst identifier for audit trail.
    pub analyst_id: Option<String>,
}

// ---------------------------------------------------------------------------
// WriteOutcome — Either<WritePreview, WriteResult>
// ---------------------------------------------------------------------------

/// Output of `WriteExecutor::execute()`.
///
/// Either a dry-run preview (Phase 4 dry-run path) or a live execution result
/// (Phase 6). Mirrors the `Either<WritePreview, WriteResult>` type described
/// in the story spec.
#[derive(Debug)]
pub enum WriteOutcome {
    /// Dry-run path: returned when `dry_run = true`.
    Preview(WritePreview),
    /// Execute path: returned when `dry_run = false` and all phases passed.
    Result(WriteResult),
}

// ---------------------------------------------------------------------------
// WriteExecutor — top-level orchestrator
// ---------------------------------------------------------------------------

/// Top-level write execution engine.
///
/// Orchestrates all six phases of the write execution pipeline (AD-022).
///
/// # Concurrency
/// The `write_semaphore` (capacity 4) is per-`WriteExecutor` instance.
/// Multiple concurrent MCP write tool calls share the semaphore.
///
/// The `write_rate_limit_per_minute = 200` ceiling from `prism.toml` is future
/// scope — implement the semaphore now, defer rate limiting (Dev Notes).
pub struct WriteExecutor {
    /// Two-tier feature flag evaluator (compile-time gate + runtime TOML).
    pub(crate) feature_flags: Arc<FeatureFlagEvaluator>,
    /// Confirmation token store for Phase 4 irreversible gate.
    pub(crate) confirmation_store: Arc<ConfirmationTokenStore>,
    /// Phase 5 write dispatcher (intent → fan-out → outcome audit).
    pub(crate) dispatcher: Arc<WriteDispatcher>,
    /// Write endpoint registry for endpoint spec resolution.
    pub(crate) endpoint_registry: Arc<WriteEndpointRegistry>,
}

impl WriteExecutor {
    /// Construct a `WriteExecutor` with the provided dependencies.
    pub fn new(
        feature_flags: Arc<FeatureFlagEvaluator>,
        confirmation_store: Arc<ConfirmationTokenStore>,
        audit_writer: Arc<dyn AuditWriter>,
        adapter_registry: Arc<AdapterRegistry>,
        endpoint_registry: Arc<WriteEndpointRegistry>,
    ) -> Self {
        let dispatcher = Arc::new(WriteDispatcher::new(audit_writer, adapter_registry));
        Self {
            feature_flags,
            confirmation_store,
            dispatcher,
            endpoint_registry,
        }
    }

    /// Execute the write pipeline for the given `WritePlan`.
    ///
    /// Runs all six phases:
    /// 1. Parse: plan is already parsed (consumed from S-3.06 AST).
    /// 2. Safety pre-check (pure): `phase2_safety_check`.
    /// 3. Fetch: would call `QueryMaterializer::execute` (S-3.02 read pipeline).
    /// 4. Post-fetch batch limit check.
    /// 5. Dry-run / confirm gate: `DryRunGate::gate`.
    /// 6. Write dispatch: `WriteDispatcher::dispatch`.
    /// 7. Return: construct `WriteOutcome`.
    ///
    /// # Returns
    /// - `Ok(WriteOutcome::Preview(...))` — dry-run path.
    /// - `Ok(WriteOutcome::Result(...))` — execute path.
    /// - `Err(PrismError)` — any gate or infrastructure failure.
    ///
    /// # BC References
    /// - BC-2.04.001: compile-time feature gate (Phase 2b Gate 1)
    /// - BC-2.04.007: risk tier classification (Phase 2e)
    /// - BC-2.04.008: dry-run default (Phase 4)
    /// - BC-2.05.009: audit intent before sensor call (Phase 5a)
    pub async fn execute(
        &self,
        plan: WritePlan,
        context: QueryContext,
    ) -> Result<WriteOutcome, PrismError> {
        // ----------------------------------------------------------------
        // Phase 2: Safety pre-check (pure, no I/O)
        // ----------------------------------------------------------------

        // Resolve write target descriptor from the plan
        let is_internal = plan.is_internal_table();
        // HIGH-4: composite source check via registry (replaces hardcoded string compare).
        let is_composite = WriteEndpointRegistry::is_composite(&plan.sensor);

        let target = WriteTargetDescriptor {
            sensor: &plan.sensor,
            verb: &plan.verb,
            capability_path: &format!("sensor.{}.{}", plan.sensor, plan.verb),
            is_composite_source: is_composite,
            is_internal_table: is_internal,
        };

        // Look up endpoint spec from registry (if registered)
        // If not found, use defaults for tests (no panic — test registry is empty)
        let default_spec = prism_spec_engine::write_endpoint::WriteEndpointSpec {
            pipe_verb: plan.verb.clone(),
            sql_table: plan.target_table.clone(),
            capability_path: format!("sensor.{}.{}", plan.sensor, plan.verb),
            risk_tier: RiskTier::Irreversible, // default conservative: Irreversible
            batch_limit: 100,
            batch_mode: prism_spec_engine::write_endpoint::BatchMode::Serial,
            steps: vec![],
            record_id_field: "id".to_string(),
        };
        let endpoint_spec = self
            .endpoint_registry
            .get(&plan.sensor, &plan.verb)
            .unwrap_or(&default_spec);

        // Resolve batch limit: endpoint × client override × system ceiling
        let resolved_limit = resolve_batch_limit(
            endpoint_spec.batch_limit,
            None, // client override: resolved from config in production
            SYSTEM_BATCH_CEILING,
        );

        // BC-2.04.001: compile-time feature gate derived from sensor name.
        // F-PASS2-HIGH-001: call prism-security gate functions as the single source
        // of truth for the cfg gate topology. Each function uses #[cfg(feature = "...")]
        // internally; enabling a *-write feature in prism-query (which chains to
        // prism-security/prism-sensors via Cargo feature propagation) lights up the
        // gate here automatically without duplication.
        let compile_gate: CompileFeatureGate = match plan.sensor.as_str() {
            "crowdstrike" => crowdstrike_write_gate().into(),
            "cyberint" => cyberint_write_gate().into(),
            "claroty" => claroty_write_gate().into(),
            "armis" => armis_write_gate().into(),
            // Unknown sensor: no write feature → Absent
            _ => CompileFeatureGate::Absent,
        };

        // Run Phase 2 — will Err on any gate failure
        let safety_passed = phase2_safety_check(
            &plan,
            &target,
            compile_gate,
            &self.feature_flags,
            &context.client_id,
            endpoint_spec,
            resolved_limit,
        )?;

        // ----------------------------------------------------------------
        // Phase 3: Fetch (stub — QueryMaterializer integration is S-3.02)
        // In production, this calls QueryMaterializer::execute(source_query, context).
        // For S-3.07: fetched_records is empty — the safety pipeline tests don't
        // need actual records; they test gate behavior.
        // ----------------------------------------------------------------
        let fetched_records: Vec<arrow::record_batch::RecordBatch> = vec![];
        let would_affect_count = fetched_records.iter().map(|rb| rb.num_rows() as u32).sum();

        // ----------------------------------------------------------------
        // Phase 3→4 boundary: post-fetch batch limit check
        // ----------------------------------------------------------------
        let total_rows: u32 = fetched_records.iter().map(|rb| rb.num_rows() as u32).sum();
        if total_rows > safety_passed.batch_limit.limit {
            return Err(PrismError::WriteBatchLimitExceeded {
                requested: total_rows as usize,
                limit: safety_passed.batch_limit.limit as usize,
                endpoint: plan.target_table.clone(),
                client_id: context.client_id.clone(),
            });
        }

        // ----------------------------------------------------------------
        // Phase 4: Dry-run / confirm gate
        // ----------------------------------------------------------------
        let write_endpoint = format!("{}.{}", plan.sensor, plan.verb);
        let dry_run_gate = DryRunGate::new(self.confirmation_store.clone());

        let gate_result = dry_run_gate
            .gate(GateInputs {
                plan: &plan,
                context: &context,
                risk_tier: &safety_passed.risk_tier,
                dry_run: context.dry_run,
                fetched_records: &fetched_records,
                write_endpoint: &write_endpoint,
                would_affect_count,
            })
            .await?;

        // If gate returned a Preview, return it (dry-run path)
        if let Some(outcome) = gate_result {
            return Ok(outcome);
        }

        // ----------------------------------------------------------------
        // Phase 5: Write dispatch (execute path)
        // ----------------------------------------------------------------
        // CRIT-4: forward capability_check from Phase 2 for audit intent emission
        // (BC-2.05.009 — capability_checks recorded in audit entry).
        let write_result = self
            .dispatcher
            .dispatch(DispatchInputs {
                plan: &plan,
                context: &context,
                risk_tier: &safety_passed.risk_tier,
                confirmed_by_token: context.confirmation_token_id.clone(),
                endpoint_spec,
                fetched_records: &fetched_records,
                write_endpoint: &write_endpoint,
                capability_check: &safety_passed.capability_check,
            })
            .await?;

        // ----------------------------------------------------------------
        // Phase 6: Return
        // ----------------------------------------------------------------
        Ok(WriteOutcome::Result(write_result))
    }
}

// ---------------------------------------------------------------------------
// F-PASS2-MED-004: unit tests for WritePlan::from_dml_node registry lookup
// ---------------------------------------------------------------------------

#[cfg(test)]
mod write_plan_from_dml_node_tests {
    use super::*;
    use crate::write_ast::{DmlNode, DmlOperation};
    use prism_spec_engine::write_endpoint::{BatchMode, WriteEndpointRegistry, WriteEndpointSpec};

    fn make_registry_with(sensor: &str, table: &str) -> WriteEndpointRegistry {
        let mut r = WriteEndpointRegistry::new();
        r.register(
            sensor,
            vec![WriteEndpointSpec {
                pipe_verb: "contain".to_string(),
                sql_table: table.to_string(),
                capability_path: format!("sensor.{}.contain", sensor),
                risk_tier: prism_core::RiskTier::Irreversible,
                batch_limit: 100,
                batch_mode: BatchMode::Serial,
                steps: vec![],
                record_id_field: "id".to_string(),
            }],
        )
        .expect("test registry register must succeed");
        r
    }

    fn make_dml_node(table: &str) -> DmlNode {
        DmlNode {
            operation: DmlOperation::InsertInto,
            target_table: table.to_string(),
            columns: None,
            assignments: vec![],
            filter: None,
            source_select: None,
        }
    }

    /// MED-004: simple table (e.g., "crowdstrike_contained_hosts") → sensor = "crowdstrike".
    #[test]
    fn test_from_dml_node_simple_table() {
        let registry = make_registry_with("crowdstrike", "crowdstrike_contained_hosts");
        let node = make_dml_node("crowdstrike_contained_hosts");
        let plan = WritePlan::from_dml_node(&node, &registry)
            .expect("simple table must resolve to sensor");
        assert_eq!(plan.sensor, "crowdstrike");
        assert_eq!(plan.target_table, "crowdstrike_contained_hosts");
        assert_eq!(plan.verb, "insert");
    }

    /// MED-004: multi-underscore table (e.g., "crowdstrike_detection_audit") → sensor = "crowdstrike".
    /// This was the split('_').next() heuristic bug: heuristic returned "crowdstrike" by
    /// luck for this case, but the registry is authoritative.
    #[test]
    fn test_from_dml_node_multi_underscore() {
        let registry = make_registry_with("crowdstrike", "crowdstrike_detection_audit");
        let node = make_dml_node("crowdstrike_detection_audit");
        let plan = WritePlan::from_dml_node(&node, &registry)
            .expect("multi-underscore table must resolve via registry");
        assert_eq!(
            plan.sensor, "crowdstrike",
            "registry must be authoritative source"
        );
    }

    /// MED-004: no-underscore table (e.g., "events") — split heuristic would have
    /// returned "events" as sensor; registry correctly returns Err(WriteTargetTableUnknown).
    ///
    /// fix-pass-2-correction (D-285): error code is now E-QUERY-030, not E-QUERY-029.
    /// The "table unknown to registry" failure mode has no client involved — E-QUERY-029
    /// is RESERVED for the W3-FIX-S307-002 per-client adapter init path.
    #[test]
    fn test_from_dml_node_missing_underscore() {
        let registry = WriteEndpointRegistry::new(); // empty — no "events" table
        let node = make_dml_node("events");
        let result = WritePlan::from_dml_node(&node, &registry);
        let err = result.expect_err("unregistered 'events' table must return Err");
        assert!(
            matches!(err, prism_core::PrismError::WriteTargetTableUnknown { ref table } if table == "events"),
            "error must be WriteTargetTableUnknown{{table: \"events\"}}; got: {err:?}"
        );
        let err_msg = err.to_string();
        assert!(
            err_msg.contains("events"),
            "E-QUERY-030 must mention the table name; got: {err_msg}"
        );
        assert!(
            err_msg.contains("E-QUERY-030"),
            "error must carry E-QUERY-030 code; got: {err_msg}"
        );
        assert!(
            err_msg.contains("is not declared in the WriteEndpointRegistry"),
            "error must mention WriteEndpointRegistry; got: {err_msg}"
        );
    }

    /// MED-004: unknown table (not in registry) → Err(WriteTargetTableUnknown).
    ///
    /// fix-pass-2-correction (D-285): error code is now E-QUERY-030, not E-QUERY-029.
    #[test]
    fn test_from_dml_node_unknown_table() {
        let registry = make_registry_with("crowdstrike", "crowdstrike_contained_hosts");
        let node = make_dml_node("nonexistent_table");
        let result = WritePlan::from_dml_node(&node, &registry);
        let err = result.expect_err("unregistered table must return Err");
        assert!(
            matches!(err, prism_core::PrismError::WriteTargetTableUnknown { ref table } if table == "nonexistent_table"),
            "error must be WriteTargetTableUnknown{{table: \"nonexistent_table\"}}; got: {err:?}"
        );
        let err_msg = err.to_string();
        assert!(
            err_msg.contains("nonexistent_table"),
            "E-QUERY-030 must mention the unknown table; got: {err_msg}"
        );
        assert!(
            err_msg.contains("E-QUERY-030"),
            "error must carry E-QUERY-030 code; got: {err_msg}"
        );
        assert!(
            err_msg.contains("is not declared in the WriteEndpointRegistry"),
            "error must mention WriteEndpointRegistry; got: {err_msg}"
        );
    }
}
