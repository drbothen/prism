//! Tests for parse_timestamp() multi-format fallback chain.
//!
//! Covers BC-2.01.006 (AC-3, EC-002, EC-003, TV-BC-2.01.006-001..003) and
//! BC-2.01.008 (timestamp parsing used by Armis fallback chain).
//!
//! All tests that call `parse_timestamp()` MUST fail (todo! panic) at Red Gate.
//!
//! Story: S-2.07 | BC: BC-2.01.006, BC-2.01.008
#![allow(clippy::expect_used, clippy::unwrap_used)]

use prism_sensors::adapter::SensorError;
use prism_sensors::timestamp::parse_timestamp;

// ---------------------------------------------------------------------------
// RFC 3339 format — TV-BC-2.01.006-001
// ---------------------------------------------------------------------------

/// BC-2.01.006 postcondition: RFC 3339 input is parsed correctly.
///
/// TV-BC-2.01.006-001: Valid ISO 8601 / RFC 3339 timestamp returns correct DateTime.
/// RED: parse_timestamp() is todo!() — will panic.
#[test]
fn test_BC_2_01_006_parse_rfc3339_returns_correct_datetime() {
    let result = parse_timestamp("2024-03-15T10:00:00Z");
    let dt = result.expect("RFC 3339 must parse successfully");
    assert_eq!(dt.to_rfc3339(), "2024-03-15T10:00:00+00:00");
}

/// BC-2.01.006: RFC 3339 with explicit offset is accepted.
/// RED: parse_timestamp() is todo!() — will panic.
#[test]
fn test_BC_2_01_006_parse_rfc3339_with_offset_returns_utc() {
    let result = parse_timestamp("2024-03-15T10:00:00+00:00");
    assert!(result.is_ok(), "RFC 3339 with +00:00 offset must parse");
}

// ---------------------------------------------------------------------------
// Unix epoch format — AC-3, EC-002, TV-BC-2.01.006-002 (partial)
// ---------------------------------------------------------------------------

/// AC-3 / BC-2.01.006: Unix epoch string "1710500000" parses to the correct DateTime.
///
/// Literal spec value from AC-3: epoch 1710500000 → 2024-03-15T15:33:20Z.
/// RED: parse_timestamp() is todo!() — will panic.
#[test]
fn test_BC_2_01_006_parse_unix_epoch_1710500000_returns_correct_datetime() {
    let result = parse_timestamp("1710500000");
    let dt = result.expect("Unix epoch string must parse successfully (AC-3)");
    // 1710500000 seconds since Unix epoch = 2024-03-15T15:33:20 UTC
    assert_eq!(
        dt.timestamp(),
        1_710_500_000,
        "Parsed DateTime must have Unix timestamp == 1710500000 (AC-3, EC-002)"
    );
}

/// BC-2.01.006: Unix epoch zero parses as the Unix epoch origin.
/// RED: parse_timestamp() is todo!() — will panic.
#[test]
fn test_BC_2_01_006_parse_unix_epoch_zero_returns_epoch_origin() {
    let result = parse_timestamp("0");
    let dt = result.expect("Unix epoch 0 must parse");
    assert_eq!(dt.timestamp(), 0);
}

/// BC-2.01.006: Negative Unix epoch (before 1970) parses correctly.
/// RED: parse_timestamp() is todo!() — will panic.
#[test]
fn test_BC_2_01_006_parse_negative_unix_epoch_parses_correctly() {
    let result = parse_timestamp("-1000");
    let dt = result.expect("Negative Unix epoch must parse");
    assert_eq!(dt.timestamp(), -1000);
}

// ---------------------------------------------------------------------------
// Custom format "%Y-%m-%dT%H:%M:%S" — TV-BC-2.01.006-002 (4th format)
// ---------------------------------------------------------------------------

/// BC-2.01.006 postcondition: Custom no-timezone format "%Y-%m-%dT%H:%M:%S" parsed as UTC.
///
/// TV-BC-2.01.006-002: Cyberint custom format succeeds.
/// RED: parse_timestamp() is todo!() — will panic.
#[test]
fn test_BC_2_01_006_parse_custom_no_tz_format_returns_utc_datetime() {
    let result = parse_timestamp("2024-03-15T10:00:00");
    let dt = result.expect("Custom no-timezone format must parse as UTC");
    assert_eq!(
        dt.timestamp(),
        1_710_496_800,
        "2024-03-15T10:00:00 UTC = Unix epoch 1710496800"
    );
}

// ---------------------------------------------------------------------------
// Error path — EC-003, TV-BC-2.01.006-003
// ---------------------------------------------------------------------------

/// EC-003 / TV-BC-2.01.006-003: Input that fails all formats returns
/// SensorError::UnparseableTimestamp, NOT a panic.
///
/// BC-2.01.006 postcondition: "Return Err(SensorError::UnparseableTimestamp { raw })
/// if all formats fail."
/// RED: parse_timestamp() is todo!() — will panic.
#[test]
fn test_BC_2_01_006_rejects_unparseable_timestamp_with_error() {
    let raw = "not-a-timestamp-at-all!!";
    let result = parse_timestamp(raw);
    match result {
        Err(SensorError::UnparseableTimestamp { raw: r }) => {
            assert_eq!(
                r, raw,
                "UnparseableTimestamp must preserve the original raw string (EC-003)"
            );
        }
        Ok(_) => panic!("Expected Err(UnparseableTimestamp) but got Ok"),
        Err(e) => panic!("Expected UnparseableTimestamp, got: {e}"),
    }
}

/// BC-2.01.006: Empty string fails all formats and returns UnparseableTimestamp.
/// RED: parse_timestamp() is todo!() — will panic.
#[test]
fn test_BC_2_01_006_rejects_empty_string_with_unparseable_timestamp() {
    let result = parse_timestamp("");
    assert!(
        matches!(result, Err(SensorError::UnparseableTimestamp { .. })),
        "Empty string must produce UnparseableTimestamp"
    );
}

/// BC-2.01.006: Partial datetime string with no time component fails gracefully.
/// RED: parse_timestamp() is todo!() — will panic.
#[test]
fn test_BC_2_01_006_rejects_date_only_string_with_unparseable_timestamp() {
    let result = parse_timestamp("2024-03-15");
    assert!(
        matches!(result, Err(SensorError::UnparseableTimestamp { .. })),
        "Date-only string must produce UnparseableTimestamp"
    );
}

/// BC-2.01.006: Non-numeric string that looks like a number but isn't fails gracefully.
/// RED: parse_timestamp() is todo!() — will panic.
#[test]
fn test_BC_2_01_006_rejects_float_string_with_unparseable_timestamp() {
    // "1710500000.5" — not a pure integer, not RFC3339, not custom format
    let result = parse_timestamp("1710500000.5");
    assert!(
        matches!(result, Err(SensorError::UnparseableTimestamp { .. })),
        "Float-looking string should fail unix-epoch (not integer) and return UnparseableTimestamp"
    );
}

// ---------------------------------------------------------------------------
// Format priority — RFC 3339 wins over Unix epoch for ambiguous-looking inputs
// ---------------------------------------------------------------------------

/// BC-2.01.006 invariant: RFC 3339 is tried first; if it succeeds, Unix epoch
/// branch is never reached.
/// RED: parse_timestamp() is todo!() — will panic.
#[test]
fn test_BC_2_01_006_rfc3339_takes_priority_over_unix_epoch() {
    // A valid RFC 3339 string — must not be interpreted as an epoch even if
    // it superficially resembles one after stripping chars.
    let result = parse_timestamp("2024-03-15T10:00:00Z");
    let dt = result.expect("RFC 3339 must succeed on first format attempt");
    // Epoch for 2024-03-15T10:00:00Z is 1710496800 — verify correct parse
    assert_eq!(dt.timestamp(), 1_710_496_800);
}
