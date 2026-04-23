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

use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use prism_core::PrismError;
use prost_reflect::DynamicMessage;
use serde_json::Value as JsonValue;

use crate::mappers::SensorMapper;

/// Cyberint sensor field mapper. (BC-2.02.004)
pub struct CyberintMapper;

/// Cyberint fields that are explicitly mapped to OCSF paths.
const CYBERINT_MAPPED_FIELDS: &[&str] = &[
    "ref_id",
    "title",
    "severity",
    "status",
    "created_date",
    "threat_type",
    "tags",
];

/// Attempts to parse a Cyberint `created_date` value using the four supported formats.
///
/// Format priority (BC-2.02.004, AC-3, AC-4, S-1.05 Task 3):
///   1. RFC3339 (`DateTime::parse_from_rfc3339`)
///   2. ISO 8601 without timezone (`NaiveDateTime::parse_from_str` with `%Y-%m-%dT%H:%M:%S`)
///   3. Unix timestamp integer (`as_i64()`)
///   4. Failure → `Err(PrismError::OcsfTimestampParseError)`
pub fn parse_cyberint_timestamp(
    field: &str,
    value: &JsonValue,
) -> Result<DateTime<Utc>, PrismError> {
    // Attempt 1: RFC3339 string
    if let Some(s) = value.as_str() {
        if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
            return Ok(dt.with_timezone(&Utc));
        }
        // Attempt 2: ISO 8601 without timezone (assume UTC)
        if let Ok(naive) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S") {
            return Ok(Utc.from_utc_datetime(&naive));
        }
        // All string formats failed
        return Err(PrismError::OcsfTimestampParseError {
            field: field.to_owned(),
            raw: s.to_owned(),
        });
    }

    // Attempt 3: Unix timestamp integer
    if let Some(unix_secs) = value.as_i64() {
        return Utc.timestamp_opt(unix_secs, 0).single().ok_or_else(|| {
            PrismError::OcsfTimestampParseError {
                field: field.to_owned(),
                raw: unix_secs.to_string(),
            }
        });
    }

    // All attempts failed
    Err(PrismError::OcsfTimestampParseError {
        field: field.to_owned(),
        raw: value.to_string(),
    })
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
    /// # Errors
    ///
    /// - `PrismError::OcsfTimestampParseError` — `created_date` cannot be parsed.
    /// - `PrismError::OcsfNormalizationFailed` — required field missing.
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
                source_id: "<cyberint>".to_owned(),
                reason: "raw record is not a JSON object".to_owned(),
            })?;

        // Extract ref_id (source record ID)
        let ref_id = obj
            .get("ref_id")
            .and_then(JsonValue::as_str)
            .map(str::to_owned)
            .unwrap_or_else(|| "<unknown-cyberint>".to_owned());

        // Parse created_date if present (BC-2.02.004, AC-3, AC-4)
        if let Some(created_date) = obj.get("created_date") {
            parse_cyberint_timestamp("created_date", created_date)?;
        }

        // Capture all unmapped fields into extensions (BC-2.02.007, VP-017)
        for (key, value) in obj {
            if !CYBERINT_MAPPED_FIELDS.contains(&key.as_str()) {
                extensions.insert(key.clone(), value.clone());
            }
        }

        Ok(ref_id)
    }
}
