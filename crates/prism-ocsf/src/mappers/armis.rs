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

use chrono::{DateTime, TimeZone, Utc};
use prism_core::PrismError;
use prost_reflect::DynamicMessage;
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

/// Armis device fields mapped to OCSF paths.
const ARMIS_DEVICE_MAPPED: &[&str] = &[
    "id",
    "name",
    "ipAddress",
    "macAddress",
    "type",
    "manufacturer",
    "last_seen",
    "created_at",
    "timestamp",
    "armis_aql_meta",
];

/// Armis alert fields mapped to OCSF paths.
const ARMIS_ALERT_MAPPED: &[&str] = &[
    "alertId",
    "type",
    "severity",
    "last_seen",
    "created_at",
    "timestamp",
    "armis_aql_meta",
];

/// General timestamp + AQL fields for other record types.
const ARMIS_COMMON_MAPPED: &[&str] = &[
    "id",
    "last_seen",
    "created_at",
    "timestamp",
    "armis_aql_meta",
];

/// Attempts to parse an RFC3339 string from an Armis timestamp field.
fn parse_armis_timestamp_str(s: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
        .or_else(|| {
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S")
                .ok()
                .map(|naive| Utc.from_utc_datetime(&naive))
        })
}

/// Extracts a timestamp from an Armis record using the fallback chain.
///
/// Fallback chain (BC-2.02.006, AC-6, S-1.05 Task 5):
///   1. `last_seen`
///   2. `created_at`
///   3. `timestamp`
///   4. Returns `Utc::now()` and emits a `tracing::warn!()` — NEVER returns an error.
pub fn extract_armis_timestamp(raw: &JsonValue) -> DateTime<Utc> {
    let obj = match raw.as_object() {
        Some(o) => o,
        None => {
            tracing::warn!("armis: raw record is not an object; using current time as fallback");
            return Utc::now();
        }
    };

    // Fallback chain: last_seen → created_at → timestamp → current time
    for field in &["last_seen", "created_at", "timestamp"] {
        if let Some(val) = obj.get(*field) {
            if let Some(s) = val.as_str() {
                if let Some(dt) = parse_armis_timestamp_str(s) {
                    return dt;
                }
            }
            // Integer unix timestamp
            if let Some(unix) = val.as_i64() {
                if let Some(dt) = Utc.timestamp_opt(unix, 0).single() {
                    return dt;
                }
            }
        }
    }

    tracing::warn!(
        "armis: no valid timestamp found in record (tried last_seen, created_at, timestamp); \
         using current time as fallback"
    );
    Utc::now()
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
    /// # Note on `msg` population (S-1.04 Red Gate constraint)
    ///
    /// `msg` is currently unused (`_msg`) — see `CrowdStrikeMapper::map()` for the
    /// full explanation. Once `ocsf-proto-gen` ships, `device.uid`, `device.name`,
    /// `device.ip`, `finding_info.uid`, `severity_id`, `category_name`, and `time`
    /// will be written to `msg`.
    fn map(
        &self,
        record_type: &str,
        raw: &serde_json::Value,
        _msg: &mut DynamicMessage,
        extensions: &mut serde_json::Map<String, serde_json::Value>,
    ) -> Result<String, PrismError> {
        let obj = raw
            .as_object()
            .ok_or_else(|| PrismError::OcsfNormalizationFailed {
                source_id: "<armis>".to_owned(),
                reason: "raw record is not a JSON object".to_owned(),
            })?;

        // Determine which fields are mapped for this record type
        let mapped_fields: &[&str] = match record_type {
            "device" => ARMIS_DEVICE_MAPPED,
            "alert" => ARMIS_ALERT_MAPPED,
            _ => ARMIS_COMMON_MAPPED,
        };

        // Extract source record ID: alertId for alerts, id for everything else
        let source_id = if record_type == "alert" {
            obj.get("alertId")
                .and_then(|v| {
                    v.as_str()
                        .map(str::to_owned)
                        .or_else(|| v.as_i64().map(|n| n.to_string()))
                })
                .unwrap_or_else(|| "<armis-alert>".to_owned())
        } else {
            obj.get("id")
                .and_then(|v| {
                    v.as_str()
                        .map(str::to_owned)
                        .or_else(|| v.as_i64().map(|n| n.to_string()))
                })
                .unwrap_or_else(|| "<armis-unknown>".to_owned())
        };

        // Forward AQL metadata if present (BC-2.02.006)
        if let Some(aql_meta) = obj.get("armis_aql_meta") {
            extensions.insert("armis_aql_meta".to_owned(), aql_meta.clone());
        }

        // Capture all unmapped fields into extensions (BC-2.02.007, VP-017)
        for (key, value) in obj {
            if !mapped_fields.contains(&key.as_str()) {
                extensions.insert(key.clone(), value.clone());
            }
        }

        Ok(source_id)
    }
}
