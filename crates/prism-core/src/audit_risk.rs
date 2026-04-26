//! AuditRiskLevel — operational audit severity for write operation classification.
//!
//! Introduced by S-2.04 (prism-audit) for use in `WriteAuditDetail.risk_tier`.
//!
//! # Type disambiguation
//!
//! `AuditRiskLevel` (Low | Medium | High | Critical) classifies operational audit
//! severity and is embedded in `WriteAuditDetail`. It is distinct from
//! `RiskTier` (Reversible | Irreversible, S-1.13) which classifies whether a
//! write endpoint's action can be reversed and drives confirmation-token
//! requirements in prism-security. The two types coexist and must NOT be
//! conflated. Never use `RiskTier` inside `WriteAuditDetail`.

use serde::{Deserialize, Serialize};

/// Operational audit severity for write operation classification (S-2.04, BC-2.05.004).
///
/// Used in `WriteAuditDetail.risk_tier` to record how risky an audited write
/// operation is from an operational security standpoint.
///
/// This is NOT the same as `prism_core::RiskTier` (reversibility tier for
/// confirmation tokens). See module-level doc for disambiguation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditRiskLevel {
    /// Low operational risk — routine read-adjacent write (e.g., tag annotation).
    Low,
    /// Medium operational risk — notable write with bounded blast radius.
    Medium,
    /// High operational risk — significant write requiring elevated attention.
    High,
    /// Critical operational risk — potentially irreversible or wide-blast-radius write.
    Critical,
}
