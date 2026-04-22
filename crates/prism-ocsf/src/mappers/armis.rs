//! Armis Centrix field mapper — maps Armis records to OCSF across 7 data sources.
//!
//! # Contract: BC-2.02.006
//!
//! Postcondition: Armis Centrix 7 data sources map to OCSF with AQL result forwarding:
//!   - Supported record types: `device`, `alert`, `activity`, `vulnerability`,
//!     `policy_violation`, `user`, `network_segment`
//!   - AQL query forwarding: embed AQL metadata into `extensions["armis_aql_meta"]`
//!   - Timestamp fallback chain (try in order for all record types):
//!       1. `last_seen`
//!       2. `created_at`
//!       3. `timestamp`
//!       4. None present → use current time + log warning (NEVER fail on missing time). (AC-6)
//!   - Device: `id` → `device.uid` (int→string), `name` → `device.name`,
//!     `ipAddress` → `device.ip`, `macAddress` → `device.mac_addr`,
//!     `type` → `device.type`, `manufacturer` → `device.hw_info.manufacturer`
//!   - Alert: `alertId` → `finding_info.uid`, `type` → `category_name`,
//!     `severity` → `severity_id`
//!   - All unmapped fields → `extensions`
//!
//! # Stub Status (S-1.05 Red Gate)
//!
//! `map()` and `extract_armis_timestamp()` bodies are `unimplemented!()`.

use chrono::{DateTime, Utc};
use prost_reflect::DynamicMessage;
use prism_core::PrismError;
use serde_json::Value as JsonValue;

use crate::mappers::SensorMapper;

/// Armis Centrix sensor field mapper. (BC-2.02.006)
pub struct ArmisMapper;

/// Supported Armis record types. (BC-2.02.006)
pub const ARMIS_RECORD_TYPES: &[&str] = &[
    "device",
    "alert",
    "activity",
    "vulnerability",
    "policy_violation",
    "user",
    "network_segment",
];

/// Extracts a timestamp from an Armis record using the fallback chain.
///
/// Fallback chain (BC-2.02.006, AC-6, S-1.05 Task 5):
///   1. `last_seen`
///   2. `created_at`
///   3. `timestamp`
///   4. Returns `Utc::now()` and emits a `tracing::warn!()` — NEVER returns an error.
///
/// # Stub — body unimplemented (S-1.05 Red Gate).
pub fn extract_armis_timestamp(_raw: &JsonValue) -> DateTime<Utc> {
    unimplemented!("extract_armis_timestamp — S-1.05 stub")
}

impl SensorMapper for ArmisMapper {
    fn sensor_id(&self) -> &'static str {
        "armis"
    }

    fn record_types(&self) -> &'static [&'static str] {
        ARMIS_RECORD_TYPES
    }

    /// Maps an Armis record to OCSF.
    ///
    /// Returns `alertId` (for alerts) or `id` (for other types) as the source record ID.
    /// Missing timestamp falls back to current time with a warning — never fails. (AC-6)
    ///
    /// # Stub — body unimplemented (S-1.05 Red Gate).
    fn map(
        &self,
        _record_type: &str,
        _raw: &serde_json::Value,
        _msg: &mut DynamicMessage,
        _extensions: &mut serde_json::Map<String, serde_json::Value>,
    ) -> Result<String, PrismError> {
        unimplemented!("ArmisMapper::map — S-1.05 stub")
    }
}
