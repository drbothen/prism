//! Phase 2 Safety Pre-check — pure, synchronous, no I/O.
//!
//! Validates a `WritePlan` against all safety gates before any fetch or
//! sensor API contact is initiated (story §Phase 2):
//!
//! a. Write target validation — rejects composite sources and internal tables.
//! b. Compile-time feature flag gate (Gate 1) — BC-2.04.001.
//! c. Runtime TOML capability gate (Gate 2) — BC-2.04.001, BC-2.05.009.
//! d. Unbounded write pre-check — E-QUERY-022.
//! e. Batch limit structural pre-check — E-QUERY-021.
//! f. Risk tier classification — AD-022, BC-2.04.007.
//!
//! # Architecture Compliance
//! - Phase 2 MUST be entirely pure — no `async fn`, no I/O.
//! - All inputs are passed synchronously; the function returns a `Result`.
//! - Audit intent is emitted at the Phase 2→3 boundary by the caller
//!   (`WriteExecutor`), NOT inside Phase 2 (BC-2.05.009).
//!
//! Story: S-3.07 | BCs: BC-2.04.001, BC-2.04.007, BC-2.05.009

use prism_core::{PrismError, RiskTier};
use prism_security::feature_flag::{CapabilityCheckResult, CompileTimeGate, FeatureFlagEvaluator};
use prism_spec_engine::write_endpoint::WriteEndpointSpec;

use crate::write_ast::DmlOperation;
use crate::write_pipeline::WritePlan;

// ---------------------------------------------------------------------------
// SafetyPreCheck input context
// ---------------------------------------------------------------------------

/// Resolved batch limit for a write operation.
///
/// `resolve_batch_limit` combines endpoint spec, per-client override, and
/// system ceiling to produce the effective limit.
#[derive(Debug, Clone, Copy)]
pub struct ResolvedBatchLimit {
    /// Effective maximum records per batch after resolution.
    pub limit: u32,
}

/// Gate 1 compile-time feature presence — modeled as a runtime enum for
/// testability (VP-020 feasibility note: "modeled as runtime bool in test").
///
/// In production the calling code is `#[cfg(feature = "...")]`-gated, so if
/// the feature is absent the call site never reaches here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompileFeatureGate {
    /// The `{sensor}-write` cargo feature is compiled into this binary.
    Present,
    /// The `{sensor}-write` cargo feature is absent — write code does not exist.
    Absent,
}

impl From<CompileFeatureGate> for CompileTimeGate {
    fn from(g: CompileFeatureGate) -> CompileTimeGate {
        match g {
            CompileFeatureGate::Present => CompileTimeGate::Present,
            CompileFeatureGate::Absent => CompileTimeGate::Absent,
        }
    }
}

// ---------------------------------------------------------------------------
// Safety check output
// ---------------------------------------------------------------------------

/// Output of a successful Phase 2 safety pre-check.
///
/// Carries the resolved risk tier and batch limit forward to Phase 3/4.
/// All gates passed — the caller may proceed to fetch phase.
#[derive(Debug, Clone)]
pub struct SafetyCheckPassed {
    /// Risk tier resolved for this write plan (BC-2.04.007).
    pub risk_tier: RiskTier,
    /// Resolved batch limit (endpoint spec × client override × system ceiling).
    pub batch_limit: ResolvedBatchLimit,
    /// The capability check result, forwarded for audit intent emission
    /// (BC-2.05.009 — emitted at Phase 2→3 boundary by caller).
    pub capability_check: CapabilityCheckResult,
}

// ---------------------------------------------------------------------------
// Phase 2 — pure safety pre-check
// ---------------------------------------------------------------------------

/// Composite write source descriptor used in Gate 1/2 validation.
///
/// Carries the write target information extracted from the parsed `WritePlan`.
#[derive(Debug, Clone)]
pub struct WriteTargetDescriptor<'a> {
    /// Sensor name (e.g., `"crowdstrike"`).
    pub sensor: &'a str,
    /// Verb name (e.g., `"contain"`).
    pub verb: &'a str,
    /// Dot-path capability identifier from the endpoint spec.
    pub capability_path: &'a str,
    /// Whether the source is composite (e.g., `EVENTS`).
    pub is_composite_source: bool,
    /// Whether the target table is an internal `prism_*` table.
    pub is_internal_table: bool,
}

/// Perform the Phase 2 safety pre-check.
///
/// Pure function — no I/O, no async. All required inputs are passed as arguments.
///
/// # Gates
/// 1. Composite source check → `E-QUERY-020`
/// 2. Internal table check → `E-QUERY-027`
/// 3. Compile-time feature gate → `E-FLAG-002` (via CapabilityDenied)
/// 4. Runtime TOML capability gate → `E-FLAG-001` (CapabilityDenied)
/// 5. Unbounded write check → `E-QUERY-022`
/// 6. Batch limit structural check → `E-QUERY-021`
/// 7. Risk tier classification (pure; DELETE FROM always Irreversible per AD-022)
///
/// # Returns
/// `Ok(SafetyCheckPassed)` if all gates pass; `Err(PrismError)` at the first
/// failing gate.
///
/// # BC References
/// - BC-2.04.001: compile-time gate
/// - BC-2.04.007: risk tier classification
/// - BC-2.05.009: capability check result forwarded for audit (emitted by caller)
pub fn phase2_safety_check(
    plan: &WritePlan,
    target: &WriteTargetDescriptor<'_>,
    compile_gate: CompileFeatureGate,
    evaluator: &FeatureFlagEvaluator,
    client_id: &str,
    endpoint_spec: &WriteEndpointSpec,
    resolved_limit: ResolvedBatchLimit,
) -> Result<SafetyCheckPassed, PrismError> {
    // Gate 1: Composite source check — E-QUERY-020
    if target.is_composite_source {
        return Err(PrismError::WriteTargetCompositeSource {
            source_name: target.sensor.to_string(),
        });
    }

    // Gate 2: Internal prism_* table check — E-QUERY-027
    if target.is_internal_table {
        return Err(PrismError::WriteTargetingInternalTable {
            table: plan.target_table.clone(),
        });
    }

    // Gate 3: Compile-time feature flag gate (BC-2.04.001)
    let capability_check =
        evaluator.check_permission(compile_gate.into(), client_id, target.capability_path);

    // Convert denied check to PrismError
    if let Some(err) = evaluator.to_error(&capability_check) {
        return Err(err);
    }

    // Gate 4 (same check_permission call above handles both compile and runtime)
    // check_permission evaluates both tiers in one call.

    // Gate 5: Unbounded write check — E-QUERY-022
    check_unbounded_write(plan)?;

    // Gate 6: Batch limit structural check — E-QUERY-021
    check_structural_batch_limit(plan, &resolved_limit)?;

    // Gate 7: Risk tier classification — DELETE always Irreversible per AD-022
    let risk_tier = classify_risk_tier(plan, endpoint_spec);

    Ok(SafetyCheckPassed {
        risk_tier,
        batch_limit: resolved_limit,
        capability_check,
    })
}

/// Resolve the effective batch limit for a write operation.
///
/// Computes `min(endpoint.batch_limit, client_override, system_ceiling)`.
///
/// Pure — no I/O.
pub fn resolve_batch_limit(
    endpoint_batch_limit: u32,
    client_override: Option<u32>,
    system_ceiling: u32,
) -> ResolvedBatchLimit {
    let mut limit = endpoint_batch_limit;
    if let Some(override_val) = client_override {
        limit = limit.min(override_val);
    }
    limit = limit.min(system_ceiling);
    ResolvedBatchLimit { limit }
}

/// Classify the risk tier for a write operation.
///
/// For `DELETE FROM` SQL DML: always `Irreversible` regardless of spec override
/// (AD-022, story §Task 3e).
///
/// Otherwise: uses `WriteEndpointSpec.risk_tier`.
///
/// Pure — no I/O.
pub fn classify_risk_tier(plan: &WritePlan, endpoint_spec: &WriteEndpointSpec) -> RiskTier {
    // DELETE FROM is always Irreversible — regardless of spec declaration (AD-022)
    if let Some(DmlOperation::Delete) = &plan.dml_operation {
        return RiskTier::Irreversible;
    }
    // All other operations: use the spec-declared risk tier
    endpoint_spec.risk_tier.clone()
}

/// Check whether the write plan has a bounding constraint (WHERE or LIMIT).
///
/// Returns `Ok(())` if bounded; `Err(E-QUERY-022)` if unbounded.
///
/// Pure — no I/O.
pub fn check_unbounded_write(plan: &WritePlan) -> Result<(), PrismError> {
    if !plan.has_where_clause && !plan.has_explicit_limit {
        return Err(PrismError::WriteUnbounded);
    }
    Ok(())
}

/// Check whether the write plan's explicit LIMIT (if any) exceeds `batch_limit`.
///
/// Returns `Ok(())` if within bounds; `Err(E-QUERY-021)` if exceeded.
///
/// This is the STRUCTURAL (pre-fetch) batch limit check. The POST-fetch check
/// is in `write_pipeline.rs` after Phase 3 materialization.
///
/// Pure — no I/O.
pub fn check_structural_batch_limit(
    plan: &WritePlan,
    batch_limit: &ResolvedBatchLimit,
) -> Result<(), PrismError> {
    if let Some(explicit_limit) = plan.explicit_limit {
        if explicit_limit > batch_limit.limit as u64 {
            return Err(PrismError::WriteBatchLimitExceeded {
                requested: explicit_limit as usize,
                limit: batch_limit.limit as usize,
                endpoint: plan.target_table.clone(),
                client_id: String::new(), // structural check — no client context yet
            });
        }
    }
    Ok(())
}
