//! Multi-format timestamp parsing for Cyberint and Armis sensor adapters.
//!
//! Cyberint responses use inconsistent timestamp formats (ISO 8601, RFC 3339,
//! Unix epoch seconds, Cyberint custom format). Armis uses per-source fallback
//! chains over multiple candidate fields. This module provides the shared
//! `parse_timestamp()` entry point used by both adapters.
//!
//! # Formats tried (in order)
//! 1. RFC 3339 (`chrono::DateTime::parse_from_rfc3339`)
//! 2. Unix epoch seconds (parse as `i64`, convert via `DateTime::from_timestamp`)
//! 3. Custom format `"%Y-%m-%dT%H:%M:%S"` (no timezone; assumed UTC)
//!
//! # Error
//! If all formats fail, returns `SensorError::UnparseableTimestamp { raw }`.
//!
//! Story: S-2.07 | BC: BC-2.01.006, BC-2.01.008

use chrono::{DateTime, NaiveDateTime, Utc};

use crate::adapter::SensorError;

// ---------------------------------------------------------------------------
// parse_timestamp — primary entry point
// ---------------------------------------------------------------------------

/// Parses a timestamp string using a multi-format fallback chain.
///
/// Tries the following formats in order:
/// 1. RFC 3339 (`2024-03-15T10:00:00Z`)
/// 2. Unix epoch seconds (`1710500000`)
/// 3. Custom no-timezone format (`2024-03-15T10:00:00`)
///
/// Returns `Err(SensorError::UnparseableTimestamp { raw })` if all formats fail.
///
/// BC: BC-2.01.006 (AC-3), BC-2.01.008
pub fn parse_timestamp(s: &str) -> Result<DateTime<Utc>, SensorError> {
    if let Some(dt) = try_rfc3339(s) {
        return Ok(dt);
    }
    if let Some(dt) = try_unix_epoch(s) {
        return Ok(dt);
    }
    if let Some(dt) = try_custom_format(s) {
        return Ok(dt);
    }
    Err(SensorError::UnparseableTimestamp { raw: s.to_string() })
}

// ---------------------------------------------------------------------------
// Internal format helpers
// ---------------------------------------------------------------------------

/// Attempts to parse `s` as RFC 3339.
///
/// Returns `None` if the input does not conform to RFC 3339.
#[inline]
pub(crate) fn try_rfc3339(s: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

/// Attempts to parse `s` as a Unix epoch second integer (`i64`).
///
/// Returns `None` if `s` is not a valid decimal integer or the value is out
/// of the valid `DateTime` range.
#[inline]
pub(crate) fn try_unix_epoch(s: &str) -> Option<DateTime<Utc>> {
    let epoch: i64 = s.parse().ok()?;
    DateTime::from_timestamp(epoch, 0)
}

/// Attempts to parse `s` using the custom no-timezone format
/// `"%Y-%m-%dT%H:%M:%S"`, treating the result as UTC.
///
/// Returns `None` if the input does not match the format.
#[inline]
pub(crate) fn try_custom_format(s: &str) -> Option<DateTime<Utc>> {
    NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S")
        .ok()
        .map(|ndt| ndt.and_utc())
}
