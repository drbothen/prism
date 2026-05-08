//! Risk tier classification for write operations (AD-022).
//!
//! Used by prism-spec-engine WriteEndpointSpec and prism-security risk gates.
//!
//! STUB — from S-1.01 (extended for S-1.13)

use serde::{Deserialize, Serialize};

/// Risk classification for a write endpoint (AD-022).
///
/// Determines confirmation requirements and audit logging depth.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
// PRE-EXISTING-TD: two RiskTier enums coexist — this one (prism-core::RiskTier,
// Reversible | Irreversible) and prism-security::risk_tier::RiskTier (Read | Reversible | Irreversible).
// The security crate's variant adds the Read tier for gating. Consolidation deferred per
// pass-1 adversarial observation. See TD-VSDD-082 (to be filed in factory-artifacts post-merge).
pub enum RiskTier {
    /// Operation can be reversed without data loss (e.g., untag, uncontain).
    Reversible,
    /// Operation cannot be trivially reversed (e.g., contain, close_alert).
    Irreversible,
}
