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

// Stub module: all non-trivial bodies are todo!() pending implementation.
#![allow(dead_code, unused_variables)]

use std::collections::HashMap;
use std::sync::Arc;

use arrow::record_batch::RecordBatch;
use prism_core::{OrgSlug, PrismError};
use prism_security::confirmation_token::ConfirmationTokenStore;
use prism_security::feature_flag::FeatureFlagEvaluator;
use prism_sensors::AdapterRegistry;
use prism_spec_engine::write_endpoint::WriteEndpointRegistry;

use crate::write_ast::{DmlNode, DmlOperation, WriteNode};
use crate::write_dispatch::{AuditWriter, WriteDispatcher};
use crate::write_result::{WritePreview, WriteResult};

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
        // GREEN-BY-DESIGN self-check:
        // "If I include this real implementation, will the test for this function
        //  pass trivially without any implementer work?"
        // Answer: Yes — this is a non-trivial constructor that involves conditional
        // mapping from WriteNode fields. Replaced with todo!().
        todo!("S-3.07 — WritePlan::from_write_node")
    }

    /// Construct a `WritePlan` from a SQL-mode `DmlNode`.
    ///
    /// Pure constructor — no I/O.
    pub fn from_dml_node(node: &DmlNode) -> Self {
        todo!("S-3.07 — WritePlan::from_dml_node")
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
    /// 3. Fetch: `QueryMaterializer::execute` (S-3.02 read pipeline).
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
        todo!("S-3.07 — WriteExecutor::execute: six-phase pipeline orchestration")
    }

    /// Post-fetch batch limit check (between Phase 3 and Phase 4).
    ///
    /// Checks `record_batches.total_rows()` against the resolved batch limit.
    /// Returns `Err(E-QUERY-021)` if exceeded — no sensor API is contacted.
    ///
    /// Pure after the fetch — no additional I/O.
    fn check_post_fetch_batch_limit(
        fetched_records: &[RecordBatch],
        batch_limit: u32,
        write_endpoint: &str,
        client_id: &str,
    ) -> Result<u32, PrismError> {
        todo!("S-3.07 — WriteExecutor::check_post_fetch_batch_limit: E-QUERY-021")
    }
}
