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
pub enum RiskTier {
    /// Operation can be reversed without data loss (e.g., untag, uncontain).
    Reversible,
    /// Operation cannot be trivially reversed (e.g., contain, close_alert).
    Irreversible,
}
