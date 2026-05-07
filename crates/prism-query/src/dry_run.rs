//! Phase 4 Dry-Run / Confirm Gate (BC-2.04.008).
//!
//! Controls the transition from preview to execution:
//! - `dry_run = true` (default): construct `WritePreview`; issue confirmation
//!   token for Irreversible tier; return `WriteOutcome::Preview`.
//! - `dry_run = false`, Reversible: proceed to Phase 5.
//! - `dry_run = false`, Irreversible: require valid `ConfirmationToken`; consume
//!   it; proceed to Phase 5.
//!
//! # Architecture Compliance
//! - No audit INTENT record is written for dry-run executions — Phase 5 is
//!   never entered when `dry_run = true` (story §Task 5c).
//! - Token generation and consumption are the only effectful operations in Phase 4.
//! - `dry_run` comes from `QueryContext`, NOT the query string (Dev Notes).
//!
//! Story: S-3.07 | BCs: BC-2.04.007, BC-2.04.008

// Stub module: all non-trivial bodies are todo!() pending implementation.
#![allow(dead_code, unused_variables)]

use arrow::record_batch::RecordBatch;
use prism_core::{PrismError, RiskTier};
use prism_security::confirmation_token::ConfirmationTokenStore;
use std::sync::Arc;

use crate::write_pipeline::{QueryContext, WriteOutcome, WritePlan};
use crate::write_result::{ConfirmationTokenPreview, WritePreview};

// ---------------------------------------------------------------------------
// DryRunGate
// ---------------------------------------------------------------------------

/// Bundled inputs to `DryRunGate::gate` to stay within the 7-argument clippy limit.
pub struct GateInputs<'a> {
    /// Write plan for the current operation.
    pub plan: &'a WritePlan,
    /// Per-call context (dry_run flag, client_id, token).
    pub context: &'a QueryContext,
    /// Resolved risk tier from Phase 2.
    pub risk_tier: &'a RiskTier,
    /// Whether to run in dry-run mode.
    pub dry_run: bool,
    /// Records fetched in Phase 3.
    pub fetched_records: &'a [RecordBatch],
    /// Resolved write endpoint identifier string.
    pub write_endpoint: &'a str,
    /// Count of records that would be affected.
    pub would_affect_count: u32,
}

/// Phase 4 dry-run / confirmation gate.
///
/// Wraps the `ConfirmationTokenStore` needed for Phase 4b irreversible token
/// consumption and Phase 4a token generation.
pub struct DryRunGate {
    /// Token store for generating (dry-run) and consuming (execute) tokens.
    pub(crate) confirmation_store: Arc<ConfirmationTokenStore>,
}

impl DryRunGate {
    /// Construct a `DryRunGate` with a shared token store.
    pub fn new(confirmation_store: Arc<ConfirmationTokenStore>) -> Self {
        Self { confirmation_store }
    }

    /// Execute Phase 4: dry-run gate decision.
    ///
    /// # Dry-run (`dry_run = true`)
    /// - Constructs a `WritePreview` from `fetched_records`.
    /// - For `Irreversible` tier: generates a `ConfirmationToken` and embeds it
    ///   as `ConfirmationTokenPreview` in the preview.
    /// - For `Reversible` tier: no token generated.
    /// - Returns `WriteOutcome::Preview`.
    ///
    /// # Execute (`dry_run = false`)
    /// - For `Reversible` tier: returns `Ok(None)` (proceed to Phase 5).
    /// - For `Irreversible` tier: calls `consume_token(context)` to validate and
    ///   consume the token embedded in `QueryContext`. Returns `Ok(None)` on
    ///   success or `Err(PrismError::TokenNotFound | TokenExpired)` on failure.
    ///
    /// # Returns
    /// - `Ok(Some(WriteOutcome::Preview(...)))` — dry-run path, return to caller.
    /// - `Ok(None)` — execute path cleared, proceed to Phase 5.
    /// - `Err(PrismError)` — token error; Phase 5 MUST NOT be entered.
    pub async fn gate(&self, inputs: GateInputs<'_>) -> Result<Option<WriteOutcome>, PrismError> {
        todo!("S-3.07 — DryRunGate::gate: Phase 4 dry-run / confirm gate")
    }

    /// Build a `WritePreview` from fetched records.
    ///
    /// Attaches a `ConfirmationTokenPreview` when `risk_tier = Irreversible`.
    /// The `confirmation_prompt` is built from structured plan fields only
    /// (prompt injection defense — Dev Notes).
    async fn build_preview(
        &self,
        plan: &WritePlan,
        risk_tier: &RiskTier,
        context: &QueryContext,
        fetched_records: &[RecordBatch],
        write_endpoint: &str,
        would_affect_count: u32,
    ) -> Result<WritePreview, PrismError> {
        todo!("S-3.07 — DryRunGate::build_preview")
    }

    /// Generate a `ConfirmationTokenPreview` for an irreversible dry-run.
    ///
    /// Calls `ConfirmationTokenStore::generate()` and wraps the result.
    async fn generate_token_preview(
        &self,
        plan: &WritePlan,
        context: &QueryContext,
        write_endpoint: &str,
        would_affect_count: u32,
    ) -> Result<ConfirmationTokenPreview, PrismError> {
        todo!("S-3.07 — DryRunGate::generate_token_preview")
    }

    /// Validate and consume a `ConfirmationToken` for an irreversible execute call.
    ///
    /// Looks up the token ID from `QueryContext.confirmation_token_id`.
    /// - If absent: `Err(PrismError::TokenNotFound)` (E-FLAG-008).
    /// - If expired: `Err(PrismError::TokenExpired)` (E-FLAG-003).
    /// - If valid: consumes the token and returns `Ok(())`.
    async fn consume_token(&self, context: &QueryContext) -> Result<(), PrismError> {
        todo!("S-3.07 — DryRunGate::consume_token: E-FLAG-008 / E-FLAG-003")
    }
}
