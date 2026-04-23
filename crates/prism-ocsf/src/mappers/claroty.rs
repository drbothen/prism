//! Claroty xDome field mapper — maps Claroty records to OCSF across 9 data sources.
//!
//! # Contract: BC-2.02.005
//!
//! Postcondition: Claroty xDome 9 data sources map to OCSF with polymorphic ID handling:
//!   - Supported record types: `asset`, `alert`, `vulnerability`, `network_baseline`,
//!     `network_event`, `process`, `connection`, `insight`, `policy_violation`
//!   - Polymorphic ID: Claroty IDs may be integer OR string; always convert to string
//!     via `format!("claroty:{}", id_value)` for OCSF uid fields. (AC-5)
//!   - Asset: `id` → `device.uid`, `name` → `device.name`, `ip_v4` → `device.ip`,
//!     `mac_address` → `device.mac_addr`, `firmware_version` → `device.hw_info.version`,
//!     `site_name` → `device.location.name`
//!   - Alert: `id` → `finding_info.uid`, `alert_type` → `category_name`,
//!     `severity` → `severity_id`, `timestamp` → `time`
//!   - Vulnerability: `id` → `finding_info.uid`, `cve_id` → `cve.uid`,
//!     `cvss_score` → `cve.cvss.base_score`
//!   - Unknown record types → `Err(PrismError::OcsfUnknownRecordType)`
//!   - All unmapped fields → `extensions`
//!
//! Note: if this file exceeds 300 lines, split into `claroty/mod.rs` + `claroty/record_types.rs`
//! per S-1.05 Dev Notes.

use prism_core::PrismError;
use prost_reflect::DynamicMessage;
use serde_json::Value as JsonValue;

use crate::mappers::SensorMapper;

/// Claroty xDome sensor field mapper. (BC-2.02.005)
pub struct ClarotyMapper;

/// Supported Claroty record types. (BC-2.02.005)
pub const CLAROTY_RECORD_TYPES: &[&str] = &[
    "asset",
    "alert",
    "vulnerability",
    "network_baseline",
    "network_event",
    "process",
    "connection",
    "insight",
    "policy_violation",
];

/// Fields mapped for Claroty asset records.
const CLAROTY_ASSET_MAPPED: &[&str] = &[
    "id",
    "name",
    "ip_v4",
    "mac_address",
    "firmware_version",
    "site_name",
];

/// Fields mapped for Claroty alert records.
const CLAROTY_ALERT_MAPPED: &[&str] = &["id", "alert_type", "severity", "timestamp"];

/// Fields mapped for Claroty vulnerability records.
const CLAROTY_VULN_MAPPED: &[&str] = &["id", "cve_id", "cvss_score"];

/// Converts a Claroty polymorphic ID (integer or string JSON value) to the OCSF uid
/// string format `"claroty:{id}"`. (BC-2.02.005, AC-5, S-1.05 Task 4)
///
/// Uses the dev notes approach:
///   `value.as_i64().map(|n| format!("claroty:{}", n))
///       .or_else(|| value.as_str().map(|s| format!("claroty:{}", s)))`
pub fn claroty_id_to_uid(value: &JsonValue) -> Option<String> {
    value
        .as_i64()
        .map(|n| format!("claroty:{n}"))
        .or_else(|| value.as_str().map(|s| format!("claroty:{s}")))
}

impl SensorMapper for ClarotyMapper {
    fn sensor_id(&self) -> &'static str {
        "claroty"
    }

    fn record_types(&self) -> &'static [&'static str] {
        CLAROTY_RECORD_TYPES
    }

    /// Maps a Claroty record to OCSF. Returns the Claroty `id` as the source record ID.
    ///
    /// # Errors
    ///
    /// - `PrismError::OcsfUnknownRecordType` — record_type not in supported set.
    /// - `PrismError::OcsfNormalizationFailed` — required field missing.
    /// # Note on `msg` population (S-1.04 Red Gate constraint)
    ///
    /// `msg` is currently unused (`_msg`) — see `CrowdStrikeMapper::map()` for the
    /// full explanation. Once `ocsf-proto-gen` ships, `device.uid`, `device.name`,
    /// `finding_info.uid`, `severity_id`, `cve.uid`, `cve.cvss.base_score`, and
    /// `time` will be written to `msg`.
    fn map(
        &self,
        record_type: &str,
        raw: &serde_json::Value,
        _msg: &mut DynamicMessage,
        extensions: &mut serde_json::Map<String, serde_json::Value>,
    ) -> Result<String, PrismError> {
        // Check supported record types first (BC-2.02.005 error case)
        if !CLAROTY_RECORD_TYPES.contains(&record_type) {
            return Err(PrismError::OcsfUnknownRecordType {
                sensor: "claroty".to_owned(),
                record_type: record_type.to_owned(),
            });
        }

        let obj = raw
            .as_object()
            .ok_or_else(|| PrismError::OcsfNormalizationFailed {
                source_id: "<claroty>".to_owned(),
                reason: "raw record is not a JSON object".to_owned(),
            })?;

        // Determine which fields are "mapped" for this record type
        let mapped_fields: &[&str] = match record_type {
            "asset" => CLAROTY_ASSET_MAPPED,
            "alert" => CLAROTY_ALERT_MAPPED,
            "vulnerability" => CLAROTY_VULN_MAPPED,
            _ => &["id"], // For other supported types, only id is mapped
        };

        // Extract source record ID from the `id` field (polymorphic: int or string)
        let source_id = obj
            .get("id")
            .and_then(claroty_id_to_uid)
            .unwrap_or_else(|| "<claroty-unknown>".to_owned());

        // Capture all unmapped fields into extensions (BC-2.02.007, VP-017)
        for (key, value) in obj {
            if !mapped_fields.contains(&key.as_str()) {
                extensions.insert(key.clone(), value.clone());
            }
        }

        Ok(source_id)
    }
}
