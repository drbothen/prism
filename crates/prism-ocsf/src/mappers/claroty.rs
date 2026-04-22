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
//! # Stub Status (S-1.05 Red Gate)
//!
//! `map()` and `claroty_id_to_string()` bodies are `unimplemented!()`.
//!
//! Note: if this file exceeds 300 lines, split into `claroty/mod.rs` + `claroty/record_types.rs`
//! per S-1.05 Dev Notes.

use prost_reflect::DynamicMessage;
use prism_core::PrismError;
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

/// Converts a Claroty polymorphic ID (integer or string JSON value) to the OCSF uid
/// string format `"claroty:{id}"`. (BC-2.02.005, AC-5, S-1.05 Task 4)
///
/// Approach per S-1.05 Dev Notes:
///   `value.as_i64().map(|n| format!("claroty:{}", n))
///       .or_else(|| value.as_str().map(|s| format!("claroty:{}", s)))`
///
/// # Stub — body unimplemented (S-1.05 Red Gate).
pub fn claroty_id_to_uid(value: &JsonValue) -> Option<String> {
    unimplemented!("claroty_id_to_uid — S-1.05 stub, value={:?}", value)
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
    /// Returns `Err(PrismError::OcsfUnknownRecordType)` for unsupported record types.
    ///
    /// # Stub — body unimplemented (S-1.05 Red Gate).
    fn map(
        &self,
        _record_type: &str,
        _raw: &serde_json::Value,
        _msg: &mut DynamicMessage,
        _extensions: &mut serde_json::Map<String, serde_json::Value>,
    ) -> Result<String, PrismError> {
        unimplemented!("ClarotyMapper::map — S-1.05 stub")
    }
}
