//! Multi-format timestamp parsing for Cyberint and Armis sensor adapters.
// Stubs: helper functions are intentionally unused until implementation.
#![allow(dead_code)]
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

use chrono::{DateTime, Utc};

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
/// # GREEN-BY-DESIGN exception
/// The function signature and error path are trivially testable without the
/// real match body — the real multi-format logic lives inside and is stubbed.
///
/// BC: BC-2.01.006 (AC-3), BC-2.01.008
pub fn parse_timestamp(_s: &str) -> Result<DateTime<Utc>, SensorError> {
    todo!(
        "AC-3 / BC-2.01.006: implement 3-format fallback chain: \
         RFC-3339 → Unix epoch i64 → custom '%Y-%m-%dT%H:%M:%S'. \
         Return SensorError::UnparseableTimestamp if all fail."
    )
}

// ---------------------------------------------------------------------------
// Internal format helpers (GREEN-BY-DESIGN: trivial dispatch stubs)
// ---------------------------------------------------------------------------

/// Attempts to parse `s` as RFC 3339.
///
/// Returns `None` if the input does not conform to RFC 3339.
///
/// GREEN-BY-DESIGN: trivial one-liner once implemented; stubbed for Red Gate.
#[inline]
pub(crate) fn try_rfc3339(_s: &str) -> Option<DateTime<Utc>> {
    todo!("BC-2.01.006: parse_from_rfc3339 attempt")
}

/// Attempts to parse `s` as a Unix epoch second integer (`i64`).
///
/// Returns `None` if `s` is not a valid decimal integer or the value is out
/// of the valid `DateTime` range.
///
/// GREEN-BY-DESIGN: trivial once implemented; stubbed for Red Gate.
#[inline]
pub(crate) fn try_unix_epoch(_s: &str) -> Option<DateTime<Utc>> {
    todo!("BC-2.01.006 AC-3: parse as i64 then DateTime::from_timestamp")
}

/// Attempts to parse `s` using the custom no-timezone format
/// `"%Y-%m-%dT%H:%M:%S"`, treating the result as UTC.
///
/// Returns `None` if the input does not match the format.
///
/// GREEN-BY-DESIGN: trivial once implemented; stubbed for Red Gate.
#[inline]
pub(crate) fn try_custom_format(_s: &str) -> Option<DateTime<Utc>> {
    todo!("BC-2.01.006: NaiveDateTime::parse_from_str with \"%Y-%m-%dT%H:%M:%S\", then .and_utc()")
}
