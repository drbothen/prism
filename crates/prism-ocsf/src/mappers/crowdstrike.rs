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

use prism_core::PrismError;
use prost_reflect::DynamicMessage;
use serde_json::Value as JsonValue;

use crate::mappers::SensorMapper;

/// CrowdStrike sensor field mapper. (BC-2.02.003)
pub struct CrowdStrikeMapper;

/// Top-level CrowdStrike detection fields that map to known OCSF paths.
/// All other top-level fields are placed in `extensions`.
const CROWDSTRIKE_MAPPED_FIELDS: &[&str] = &[
    "detection_id",
    "severity",
    "severity_name",
    "created_timestamp",
    "device",
    "behaviors",
    "ioc_type",
    "ioc_value",
];

/// Maps a CrowdStrike severity string to an OCSF v1.x severity_id integer.
///
/// Per BC-2.02.003 and OCSF v1.x name-to-id mapping:
///   `"Informational"` → 1, `"Low"` → 2, `"Medium"` → 3, `"High"` → 4,
///   `"Critical"` → 5; unrecognized strings → 99 (Other).
///
/// Note: `"Fatal"` is OCSF id 6 per the extended BC mapping table.
pub fn crowdstrike_severity_to_id(severity: &str) -> u32 {
    match severity {
        "Informational" | "informational" => 1,
        "Low" | "low" => 2,
        "Medium" | "medium" => 3,
        "High" | "high" => 4,
        "Critical" | "critical" => 5,
        "Fatal" | "fatal" => 6,
        _ => 99,
    }
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
    /// # Errors
    ///
    /// - `PrismError::OcsfNormalizationFailed` — `detection_id` is missing.
    /// # Note on `msg` population (S-1.04 Red Gate constraint)
    ///
    /// `msg` is currently unused (`_msg`) because writing named fields to a
    /// `DynamicMessage` requires a populated `DescriptorPool` (real OCSF proto
    /// descriptors from `ocsf-proto-gen`). The pool is a stub (empty bytes) until
    /// `ocsf-proto-gen` ships (S-1.04 scope). Once available, `msg` will be populated
    /// with `severity_id`, `finding_info.uid`, `time`, `device.*`, `attacks[*]`, etc.
    /// This is tracked as tech-debt (S-1.04 Red Gate, 4 failing tests pre-existing).
    fn map(
        &self,
        _record_type: &str,
        raw: &serde_json::Value,
        _msg: &mut DynamicMessage,
        extensions: &mut serde_json::Map<String, serde_json::Value>,
    ) -> Result<String, PrismError> {
        let obj = raw
            .as_object()
            .ok_or_else(|| PrismError::OcsfNormalizationFailed {
                source_id: "<crowdstrike>".to_owned(),
                reason: "raw record is not a JSON object".to_owned(),
            })?;

        // Extract detection_id — required field (BC-2.02.003, AC-9, BC-2.02.011)
        let detection_id = obj
            .get("detection_id")
            .and_then(JsonValue::as_str)
            .ok_or_else(|| PrismError::OcsfNormalizationFailed {
                source_id: "<crowdstrike-detection>".to_owned(),
                reason: "required field 'detection_id' is missing or not a string".to_owned(),
            })?
            .to_owned();

        // Preserve severity_name in extensions (BC-2.02.003 postcondition)
        if let Some(sev_name) = obj.get("severity_name") {
            extensions.insert("crowdstrike_severity_name".to_owned(), sev_name.clone());
        }

        // Capture all unmapped top-level fields into extensions (BC-2.02.007, VP-017)
        for (key, value) in obj {
            if !CROWDSTRIKE_MAPPED_FIELDS.contains(&key.as_str()) {
                extensions.insert(key.clone(), value.clone());
            }
        }

        Ok(detection_id)
    }
}
