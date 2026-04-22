//! Cyberint field mapper — maps Cyberint alert/ioc JSON to OCSF.
//!
//! # Contract: BC-2.02.004
//!
//! Postcondition: Cyberint alert multi-format timestamps are parsed correctly:
//!   - `ref_id`       → `finding_info.uid` (source record ID)
//!   - `title`        → `finding_info.title`
//!   - `severity`     → `severity_id`
//!   - `status`       → `status_id`
//!   - `created_date` — multi-format timestamp parsing (try in order):
//!       1. RFC3339: `"2024-03-15T10:30:00Z"`
//!       2. ISO 8601 no tz: `"2024-03-15T10:30:00"` (assume UTC)
//!       3. Unix timestamp integer: `1710498600`
//!       4. If all fail → `Err(PrismError::OcsfTimestampParseError)`
//!   - `threat_type`  → `category_name`
//!   - `tags[*]`      → `labels[*]`
//!   - All unmapped   → `extensions`
//!
//! # Stub Status (S-1.05 Red Gate)
//!
//! `map()` and `parse_cyberint_timestamp()` bodies are `unimplemented!()`.

use chrono::{DateTime, Utc};
use prost_reflect::DynamicMessage;
use prism_core::PrismError;
use serde_json::Value as JsonValue;

use crate::mappers::SensorMapper;

/// Cyberint sensor field mapper. (BC-2.02.004)
pub struct CyberintMapper;

/// Attempts to parse a Cyberint `created_date` value using the four supported formats.
///
/// Format priority (BC-2.02.004, AC-3, AC-4, S-1.05 Task 3):
///   1. RFC3339 (`DateTime::parse_from_rfc3339`)
///   2. ISO 8601 without timezone (`NaiveDateTime::parse_from_str` with `%Y-%m-%dT%H:%M:%S`)
///   3. Unix timestamp integer (`as_i64()`)
///   4. Failure → `Err(PrismError::OcsfTimestampParseError)`
///
/// # Stub — body unimplemented (S-1.05 Red Gate).
pub fn parse_cyberint_timestamp(
    _field: &str,
    _value: &JsonValue,
) -> Result<DateTime<Utc>, PrismError> {
    unimplemented!("parse_cyberint_timestamp — S-1.05 stub")
}

impl SensorMapper for CyberintMapper {
    fn sensor_id(&self) -> &'static str {
        "cyberint"
    }

    fn record_types(&self) -> &'static [&'static str] {
        &["alert", "ioc"]
    }

    /// Maps a Cyberint record to OCSF. Returns `ref_id` as the source ID.
    ///
    /// # Stub — body unimplemented (S-1.05 Red Gate).
    fn map(
        &self,
        _record_type: &str,
        _raw: &serde_json::Value,
        _msg: &mut DynamicMessage,
        _extensions: &mut serde_json::Map<String, serde_json::Value>,
    ) -> Result<String, PrismError> {
        unimplemented!("CyberintMapper::map — S-1.05 stub")
    }
}
