//! Red Gate tests for S-PRISMQL-CASE-INSENSITIVE-001.
//!
//! Covers RG-019, RG-020, RG-021 — adapter-boundary OCSF enum-label canonical-case
//! normalization via `OcsfEnumMap::normalize_label`.
//!
//! Red Gate discipline (BC-5.38.001): all tests FAIL before implementation because
//! `OcsfEnumMap::normalize_label` has a `todo!()` body and panics on any invocation.
//!
//! Behavioral contracts traced:
//!   BC-2.02.013 v1.0 — Adapter-Boundary OCSF Enum-Label Canonical-Case Normalization
//!   BC-2.02.002 v1.5 — DynamicMessage Creation (normalization applied before construction)
//!   BC-2.02.010 v1.5 — OcsfEnumMap as sole canonical casing authority

#![allow(non_snake_case, clippy::expect_used)]

use crate::OcsfEnumMap;

// ─────────────────────────────────────────────────────────────────────────────
// RG-019: AC-016 — OCSF enum-label fields normalized to canonical Title-case
// ─────────────────────────────────────────────────────────────────────────────

/// RG-019: `normalize_label("severity_id", "CRITICAL")` returns `Some("Critical")`.
/// Also: `"low"` → `Some("Low")`, `"MEDIUM"` → `Some("Medium")`, `"high"` → `Some("High")`.
///
/// Red Gate: PANICS — `normalize_label` hits `todo!()` on the first call.
/// Green Gate: PASSES once normalize_label scans the inner map for case-insensitive matches.
///
/// Traces to: BC-2.02.013 v1.0 postconditions "Severity (guaranteed)";
/// BC-2.02.010 v1.5 (enum_map.rs sole casing authority).
#[test]
fn test_S_PRISMQL_CASE_INSENSITIVE_001_adapter_normalization_critical_to_title_case() {
    let map = OcsfEnumMap::new();

    // OCSF v1.x severity_id captions (from enum_map.rs new() method):
    // 0=Unknown, 1=Informational, 2=Low, 3=Medium, 4=High, 5=Critical, 99=Other

    assert_eq!(
        map.normalize_label("severity_id", "CRITICAL"),
        Some("Critical"),
        "RG-019: 'CRITICAL' must normalize to 'Critical' (OCSF severity_id=5)"
    );
    assert_eq!(
        map.normalize_label("severity_id", "low"),
        Some("Low"),
        "RG-019: 'low' must normalize to 'Low' (OCSF severity_id=2)"
    );
    assert_eq!(
        map.normalize_label("severity_id", "MEDIUM"),
        Some("Medium"),
        "RG-019: 'MEDIUM' must normalize to 'Medium' (OCSF severity_id=3)"
    );
    assert_eq!(
        map.normalize_label("severity_id", "high"),
        Some("High"),
        "RG-019: 'high' must normalize to 'High' (OCSF severity_id=4)"
    );
    assert_eq!(
        map.normalize_label("severity_id", "INFORMATIONAL"),
        Some("Informational"),
        "RG-019: 'INFORMATIONAL' must normalize to 'Informational' (OCSF severity_id=1)"
    );
    assert_eq!(
        map.normalize_label("severity_id", "unknown"),
        Some("Unknown"),
        "RG-019: 'unknown' must normalize to 'Unknown' (OCSF severity_id=0)"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// RG-020: AC-017 — Normalization is idempotent: already-canonical values unchanged
// ─────────────────────────────────────────────────────────────────────────────

/// RG-020: `normalize_label("severity_id", "High")` returns `Some("High")` — the
/// value is already in OCSF canonical Title-case and must pass through unchanged.
///
/// This is the CrowdStrike path (EC-02-020): CrowdStrike emits 'High' which is
/// already canonical, so idempotent normalization must not alter it.
///
/// Red Gate: PANICS — `normalize_label` hits `todo!()`.
/// Green Gate: PASSES — the case-insensitive scan matches 'High' = 'High',
/// returns the canonical caption 'High' unchanged.
///
/// Traces to: BC-2.02.013 v1.0 postcondition
/// "idempotent: if field already contains canonical-case value, value unchanged";
/// EC-02-020.
#[test]
fn test_S_PRISMQL_CASE_INSENSITIVE_001_adapter_normalization_idempotent_high() {
    let map = OcsfEnumMap::new();

    // CrowdStrike emits 'High' which is already canonical — must not be altered
    assert_eq!(
        map.normalize_label("severity_id", "High"),
        Some("High"),
        "RG-020: already-canonical 'High' must return Some(\"High\") unchanged (idempotent)"
    );

    // Other already-canonical values
    assert_eq!(
        map.normalize_label("severity_id", "Critical"),
        Some("Critical"),
        "RG-020: already-canonical 'Critical' must return Some(\"Critical\") unchanged"
    );
    assert_eq!(
        map.normalize_label("severity_id", "Low"),
        Some("Low"),
        "RG-020: already-canonical 'Low' must return Some(\"Low\") unchanged"
    );
    assert_eq!(
        map.normalize_label("severity_id", "Medium"),
        Some("Medium"),
        "RG-020: already-canonical 'Medium' must return Some(\"Medium\") unchanged"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// RG-021: AC-018 — Unrecognized vendor values return None (non-fatal)
// ─────────────────────────────────────────────────────────────────────────────

/// RG-021: `normalize_label("severity_id", "UNHANDLED")` returns `None` because
/// 'UNHANDLED' has no matching entry in the OCSF severity_id caption table.
///
/// Normalization is non-fatal: `normalize_label` returns `None`, never panics,
/// and never returns `Err`. The caller (adapter pipeline) logs a `tracing::warn!`
/// and passes the value through unchanged.
///
/// Red Gate: PANICS — `normalize_label` hits `todo!()`.
/// Green Gate: PASSES — unrecognized captions return `None`.
///
/// Traces to: BC-2.02.013 v1.0 error case
/// "Warning (non-fatal): value has no matching caption";
/// EC-02-021 (Armis 'UNHANDLED' vendor-specific value).
#[test]
fn test_S_PRISMQL_CASE_INSENSITIVE_001_adapter_normalization_unrecognized_value_left_as_received() {
    let map = OcsfEnumMap::new();

    // Armis vendor-specific value: no OCSF caption match → None
    assert_eq!(
        map.normalize_label("severity_id", "UNHANDLED"),
        None,
        "RG-021: 'UNHANDLED' has no OCSF severity_id caption → must return None (non-fatal)"
    );

    // Additional unrecognized values that must also return None
    assert_eq!(
        map.normalize_label("severity_id", "VENDOR_SPECIFIC_777"),
        None,
        "RG-021: any unrecognized value must return None without panic"
    );
    assert_eq!(
        map.normalize_label("severity_id", ""),
        None,
        "RG-021: empty string must return None without panic"
    );

    // Cross-field: a severity caption used as an activity_id value → None
    // (normalize_label is field-scoped; 'High' is a severity caption, not activity)
    assert_eq!(
        map.normalize_label("activity_id", "HIGH"),
        None,
        "RG-021: 'HIGH' has no activity_id caption → must return None (cross-field check)"
    );
}
