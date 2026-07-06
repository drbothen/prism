//! Shared datetime parsing utilities for sensor column normalisation.
//!
//! `parse_datetime_to_micros` is the canonical RFC-3339 → microseconds helper referenced
//! by ADR-052 D2/D5. It is shared between:
//!
//! - `prism-bin::spec_driven_adapter::build_column_array` (sensor ingestion path)
//! - `prism-query::infusion_udf::coerce_to_typed` (enrichment UDF typed output path,
//!   ADR-051 D2; S-DEMO-ENRICHMENT-TYPED-OUTPUT-001)
//!
//! **Do NOT introduce a second RFC-3339 parser.** Any new datetime-parsing site that
//! needs Timestamp(µs,UTC) output MUST call this function (ADR-052 D5).

use crate::error::SpecEngineError;

/// Parse an RFC-3339 datetime string to microseconds since the Unix epoch.
///
/// # Parameters
/// - `value`: The raw datetime string (must be RFC-3339, e.g. `"2024-01-01T00:00:00Z"`).
/// - `column_name`: The sensor/enrichment column name — used in the error struct for
///   operator diagnostics. For enrichment UDFs this is the UDF field name.
/// - `sensor_id`: The sensor or infusion identifier — used in the error struct.
///   For enrichment UDFs this is the `infusion_id`.
///
/// # Returns
/// `Ok(i64)` — microseconds since Unix epoch (UTC), suitable for storing in an Arrow
/// `Timestamp(Microsecond, Some("UTC"))` column.
///
/// `Err(SpecEngineError::TimestampParseFailure)` — the string is not valid RFC-3339.
/// Callers SHOULD emit a structured `tracing::warn!` and produce a `NULL` cell (non-fatal
/// at the row level per ADR-051/ADR-052 design decisions).
///
/// # Security (AD-017 / CWE-532)
/// The error struct captures at most 50 codepoints of `value` to prevent accidental
/// exposure of long external data in structured log lines.
///
/// # ADR-052 D5 — identical chrono strictness
/// Uses `chrono::DateTime::parse_from_rfc3339` exclusively.  The normaliser's
/// lenient-IN behaviour in the ingestion path is intentionally outside D5 scope.
pub fn parse_datetime_to_micros(
    value: &str,
    column_name: &str,
    sensor_id: &str,
) -> Result<i64, SpecEngineError> {
    chrono::DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.timestamp_micros())
        .map_err(|_| SpecEngineError::TimestampParseFailure {
            sensor_id: sensor_id.to_owned(),
            column_name: column_name.to_owned(),
            attempted_formats: vec!["rfc3339".to_owned()],
            // SEC-002 (CWE-532 / AD-017): cap raw value at 50 codepoints before storing
            // in the error struct. Display output then naturally caps — consistent with
            // E-QUERY-041/042 value_prefix convention.
            value: value.chars().take(50).collect(),
        })
}
