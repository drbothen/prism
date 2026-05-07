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

use arrow::record_batch::RecordBatch;
use prism_core::{PrismError, RiskTier};
use prism_security::confirmation_token::ConfirmationTokenStore;
use serde_json::Value;
use std::sync::Arc;
use ulid::Ulid;

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
        if inputs.dry_run {
            // Dry-run path: build preview, generate token if Irreversible
            let preview = self
                .build_preview(
                    inputs.plan,
                    inputs.risk_tier,
                    inputs.context,
                    inputs.fetched_records,
                    inputs.write_endpoint,
                    inputs.would_affect_count,
                )
                .await?;
            return Ok(Some(WriteOutcome::Preview(preview)));
        }

        // Execute path: check risk tier
        match inputs.risk_tier {
            RiskTier::Reversible => {
                // Reversible: proceed directly to Phase 5 — no token required
                // (BC-2.04.008, story §Task 5b "For Reversible tier: proceed directly")
                Ok(None)
            }
            RiskTier::Irreversible => {
                // Irreversible: require a valid ConfirmationToken (EC-04-003)
                self.consume_token(
                    inputs.context,
                    inputs.plan,
                    inputs.write_endpoint,
                    inputs.would_affect_count,
                )
                .await?;
                Ok(None)
            }
        }
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
        // Sample records: first 5 rows from the fetched batch (as JSON values).
        // We store row count in the preview; full Arrow serialization is out of scope for stub.
        let sample_records: Vec<Value> = fetched_records
            .iter()
            .take(5)
            .map(|_rb| Value::String(format!("<record batch: {} rows>", _rb.num_rows())))
            .collect();

        // Generate token for Irreversible tier
        let confirmation_token = match risk_tier {
            RiskTier::Irreversible => {
                let token_preview = self
                    .generate_token_preview(plan, context, write_endpoint, would_affect_count)
                    .await?;
                Some(token_preview)
            }
            RiskTier::Reversible => None,
        };

        // Prompt injection defense: build from structured fields only (Dev Notes)
        let confirmation_prompt = format!(
            "Would {} {} record(s) on {} for client {}. \
             This operation is {:?} and requires confirmation.",
            plan.verb, would_affect_count, write_endpoint, context.client_id, risk_tier,
        );

        Ok(WritePreview {
            query_id: Ulid::new(),
            dry_run: true, // invariant: WritePreview.dry_run is always true
            write_endpoint: write_endpoint.to_string(),
            risk_tier: risk_tier.clone(),
            would_affect_count,
            sample_records,
            reversibility: risk_tier.clone(),
            confirmation_token,
            confirmation_prompt,
        })
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
        // Build structured action params (prompt injection safe — derived from structured
        // query plan fields only, never from analyst free-text — Dev Notes).
        // Include plan.params in the hash so token is bound to the specific action params
        // (BC-2.04.012 — prevents "host-A token" from being used for "host-B" write).
        let params_json: serde_json::Value = plan
            .params
            .iter()
            .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
            .collect::<serde_json::Map<_, _>>()
            .into();
        let action_params = serde_json::json!({
            "verb": plan.verb,
            "sensor": plan.sensor,
            "target_table": plan.target_table,
            "write_endpoint": write_endpoint,
            "would_affect_count": would_affect_count,
            "client_id": context.client_id,
            "params": params_json,
        });

        let action_summary = format!(
            "{} {} record(s) on {} for client {}",
            plan.verb, would_affect_count, write_endpoint, context.client_id
        );

        let token = self.confirmation_store.generate(
            &context.client_id,
            &format!("write.{}", plan.verb),
            action_params,
            &action_summary,
        )?;

        // Convert SystemTime to DateTime<Utc> for serialization
        let expires_at = chrono::DateTime::<chrono::Utc>::from(token.expires_at);

        Ok(ConfirmationTokenPreview {
            token_id: token.token_id,
            expires_at,
            action_hash: token.action_hash,
            action_summary: token.action_summary,
        })
    }

    /// Validate and consume a `ConfirmationToken` for an irreversible execute call.
    ///
    /// Looks up the token ID from `QueryContext.confirmation_token_id`.
    /// - If absent: `Err(PrismError::TokenNotFound)` (E-FLAG-008).
    /// - If expired: `Err(PrismError::TokenExpired)` (E-FLAG-003).
    /// - If valid: consumes the token and returns `Ok(())`.
    ///
    /// The `action_params` passed to `consume()` MUST match those used at generation
    /// time (BC-2.04.012 content hash verification). We reconstruct them here from
    /// the plan so the hash matches without requiring the caller to re-supply params.
    async fn consume_token(
        &self,
        context: &QueryContext,
        plan: &WritePlan,
        write_endpoint: &str,
        would_affect_count: u32,
    ) -> Result<(), PrismError> {
        // No token ID provided in context → E-FLAG-008
        let token_id =
            context
                .confirmation_token_id
                .as_deref()
                .ok_or_else(|| PrismError::TokenNotFound {
                    token_id: String::from("<none>"),
                })?;

        // Reconstruct the same action_params used during token generation
        // (must match generate_token_preview's params for hash verification).
        // Includes plan.params so hash is bound to the specific action (BC-2.04.012).
        let params_json: serde_json::Value = plan
            .params
            .iter()
            .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
            .collect::<serde_json::Map<_, _>>()
            .into();
        let action_params = serde_json::json!({
            "verb": plan.verb,
            "sensor": plan.sensor,
            "target_table": plan.target_table,
            "write_endpoint": write_endpoint,
            "would_affect_count": would_affect_count,
            "client_id": context.client_id,
            "params": params_json,
        });

        self.confirmation_store
            .consume(token_id, &context.client_id, &action_params)
            .map(|_| ())
    }
}
