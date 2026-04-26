//! Write operation audit detail (BC-2.05.004).
//!
//! `WriteAuditDetail` is embedded in `AuditEntry.parameters` for write tool
//! invocations. It records capability check outcome, risk tier, any confirmation
//! token consumed, and the execution outcome.

use prism_core::AuditRiskLevel;
use serde::{Deserialize, Serialize};

/// Result of a capability check for a write operation (BC-2.05.004).
///
/// Note: the fine-grained `CapabilityCheckRecord` (with path/compile_time/runtime
/// fields) lives in `audit_entry` for the `capability_checks` array. This enum
/// records the aggregate grant/deny decision embedded in `WriteAuditDetail`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum CapabilityCheckResult {
    /// Capability check passed — write may proceed.
    Granted,
    /// Capability check failed — write was blocked.
    Denied {
        /// Human-readable denial reason (capability path + flag resolution).
        reason: String,
    },
}

/// Final outcome of a write operation execution (BC-2.05.004).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WriteOutcome {
    /// Write was committed to the sensor API.
    Committed,
    /// Write was rolled back (sensor or internal error during execution).
    RolledBack,
    /// Write was aborted before execution (e.g., audit persistence failed,
    /// capability denied, or confirmation token consumed but execution failed).
    Aborted,
    /// Dry-run requested — write was previewed but not executed.
    DryRun,
    /// Confirmation token was issued (first step of irreversible write).
    ConfirmationTokenIssued,
}

/// Detail record embedded in `AuditEntry.parameters` for write tool invocations.
///
/// Provides full traceability of the write access decision and execution
/// outcome for SOC 2 / ISO 27001 evidence (BC-2.05.004).
///
/// # Embedding
///
/// Serialise this struct and include it as a field in the `parameters`
/// `serde_json::Value` map before constructing `AuditEntry`.
/// Key: `"write_audit_detail"`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteAuditDetail {
    /// Result of the capability / feature-flag check before execution.
    pub capability_check: CapabilityCheckResult,

    /// Risk classification of the write operation (BC-2.05.004, S-2.04 v1.5).
    ///
    /// Uses `prism_core::AuditRiskLevel` (Low | Medium | High | Critical).
    /// NOTE: v1.4 stub used `RiskTier` here — corrected to `AuditRiskLevel` per
    /// S-2.04 v1.5 PO spec correction and Dev Notes disambiguation.
    pub risk_tier: AuditRiskLevel,

    /// Token ID of the confirmation token consumed for this write, if any.
    ///
    /// `None` for reversible (dry-run) writes and for the first step of
    /// irreversible writes (token issuance step).
    pub confirmation_token_used: Option<String>,

    /// Execution outcome (committed, rolled_back, aborted, dry_run, etc.).
    pub execution_outcome: WriteOutcome,
}

impl WriteAuditDetail {
    /// Construct a `WriteAuditDetail` for a write operation.
    pub fn new(
        capability_check: CapabilityCheckResult,
        risk_tier: AuditRiskLevel,
        confirmation_token_used: Option<String>,
        execution_outcome: WriteOutcome,
    ) -> Self {
        Self {
            capability_check,
            risk_tier,
            confirmation_token_used,
            execution_outcome,
        }
    }

    /// Serialise this detail into a `serde_json::Value` for embedding in
    /// `AuditEntry.parameters["write_audit_detail"]`.
    ///
    /// # Errors
    ///
    /// Returns `serde_json::Error` if serialisation fails (should not happen
    /// for well-formed structs).
    pub fn to_json(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::to_value(self)
    }
}

#[cfg(test)]
mod tests {}
