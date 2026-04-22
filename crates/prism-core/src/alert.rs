// S-1.02 — Alert entity types.

use serde::{Deserialize, Serialize};

/// OCSF-aligned severity level for an alert.
///
/// `as_ocsf_severity_id()` maps to OCSF `severity_id` values:
/// Critical → 5, High → 4, Medium → 3, Low → 2, Informational → 1
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AlertSeverity {
    Critical,
    High,
    Medium,
    Low,
    Informational,
}

impl AlertSeverity {
    /// Returns the OCSF `severity_id` integer for this severity level.
    ///
    /// AC-8: `AlertSeverity::Critical` → 5.
    pub fn as_ocsf_severity_id(&self) -> u32 {
        unimplemented!("implement in S-1.02 — stub for Red Gate")
    }
}
