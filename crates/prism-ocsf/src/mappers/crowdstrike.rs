//! CrowdStrike field mapper — maps CrowdStrike detection/incident/event JSON to OCSF.
//!
//! # Contract: BC-2.02.003
//!
//! Postcondition: CrowdStrike detection fields map to correct OCSF paths:
//!   - `detection_id`      → `finding_info.uid`
//!   - `severity` (string) → `severity_id` (Critical=5, High=4, Medium=3, Low=2, Informational=1)
//!   - `created_timestamp` → `time` (RFC3339)
//!   - `device.hostname`   → `device.name`
//!   - `device.device_id`  → `device.uid`
//!   - `device.local_ip`   → `device.ip`
//!   - `device.os_version` → `device.os.version`
//!   - `behaviors[*].tactic`    → `attacks[*].tactic.name`
//!   - `behaviors[*].technique` → `attacks[*].technique.name`
//!   - `ioc_type` + `ioc_value` → `evidences[0].data.type` + `evidences[0].data.value`
//!   - All other top-level fields → `extensions`
//!   - Source record ID: `detection_id`
//!
//! # Stub Status (S-1.05 Red Gate)
//!
//! `map()` body is `unimplemented!()`.

use prost_reflect::DynamicMessage;
use prism_core::PrismError;

use crate::mappers::SensorMapper;

/// CrowdStrike sensor field mapper. (BC-2.02.003)
pub struct CrowdStrikeMapper;

/// OCSF severity_id integer for each CrowdStrike severity string.
///
/// Per BC-2.02.003 and OCSF v1.x:
///   Critical=5, High=4, Medium=3, Low=2, Informational=1, unknown→99
///
/// STUB: implemented as a static lookup. Alias tables (BC-2.02.008) use phf_map!
/// in the real implementation, but a simple match is acceptable here since this
/// function is only called from `map()` which is itself unimplemented.
pub fn crowdstrike_severity_to_id(severity: &str) -> u32 {
    unimplemented!(
        "crowdstrike_severity_to_id({:?}) — S-1.05 stub",
        severity
    )
}

impl SensorMapper for CrowdStrikeMapper {
    fn sensor_id(&self) -> &'static str {
        "crowdstrike"
    }

    fn record_types(&self) -> &'static [&'static str] {
        &["detection", "incident", "event"]
    }

    /// Maps a CrowdStrike record to OCSF. Returns `detection_id` as the source ID.
    ///
    /// # Stub — body unimplemented (S-1.05 Red Gate).
    fn map(
        &self,
        _record_type: &str,
        _raw: &serde_json::Value,
        _msg: &mut DynamicMessage,
        _extensions: &mut serde_json::Map<String, serde_json::Value>,
    ) -> Result<String, PrismError> {
        unimplemented!("CrowdStrikeMapper::map — S-1.05 stub")
    }
}
