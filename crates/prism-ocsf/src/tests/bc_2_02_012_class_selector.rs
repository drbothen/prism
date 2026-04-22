//! Tests for BC-2.02.012 — OCSF Event Class Selection Per Sensor Record Type.
//!
//! BC: Each sensor record type maps deterministically to exactly one OCSF event class.
//! Security Finding (class_uid 2001) is deprecated since OCSF v1.1.0 and must not be used.
//!
//! Acceptance Criteria covered:
//! - AC-6: `EventClassSelector::select("crowdstrike", "detection")` returns `Ok(2004)`.
//! - AC-7: `EventClassSelector::select("claroty", "alert")` returns `Ok(2004)`.
//! - AC-8: `EventClassSelector::select("vendor_x", "unknown_type")` returns `Err(OcsfUnknownEventClass)`.
//!
//! Test Vectors (BC-2.02.012):
//! - TV-BC-2.02.012-001: crowdstrike_detection → 2004 (Detection Finding)
//! - TV-BC-2.02.012-002: claroty_device → 5001 (Device Inventory Info)
//! - TV-BC-2.02.012-003: claroty_vulnerability → 2002 (Vulnerability Finding)
//! - TV-BC-2.02.012-004: armis_audit_log → 3001 (Audit Activity)
//! - TV-BC-2.02.012-005: claroty_event (no OCSF mapping) → Err (stub)
//! - TV-BC-2.02.012-006: entirely unknown record type → Err
//!
//! # Note on assert patterns
//!
//! `PrismError` does not derive `PartialEq` (it is `#[non_exhaustive]` and contains
//! source-error types). Tests that check `Ok(class_uid)` use `.is_ok()` + `.unwrap()`
//! rather than `assert_eq!(result, Ok(...))`.
//!
//! # Red Gate
//!
//! All tests in this file PASS with the stub because EventClassSelector is a
//! compile-time constant mapping table — fully functional without ocsf-proto-gen.

use prism_core::PrismError;

use crate::class_selector::{
    EventClassSelector, CLASS_UID_AUDIT_ACTIVITY, CLASS_UID_DETECTION_FINDING,
    CLASS_UID_DEVICE_INVENTORY_INFO, CLASS_UID_INCIDENT_FINDING, CLASS_UID_VULNERABILITY_FINDING,
};

// Note: these tests are expected to PASS (the class selector is a lookup table,
// not gated on ocsf-proto-gen). They are kept here because they verify AC-6, AC-7, AC-8.

/// BC-2.02.012 / AC-6 / TV-BC-2.02.012-001:
/// CrowdStrike detection → Detection Finding (class_uid 2004). NOT deprecated 2001.
#[test]
fn test_BC_2_02_012_crowdstrike_detection_returns_2004() {
    let result = EventClassSelector::select("crowdstrike", "detection");
    assert!(
        result.is_ok(),
        "crowdstrike/detection must return Ok (AC-6, BC-2.02.012); got {:?}",
        result
    );
    assert_eq!(
        result.unwrap(),
        CLASS_UID_DETECTION_FINDING,
        "crowdstrike/detection must map to Detection Finding (2004), NOT deprecated 2001 (AC-6)"
    );
}

/// BC-2.02.012: CrowdStrike incident → Incident Finding (class_uid 2005).
#[test]
fn test_BC_2_02_012_crowdstrike_incident_returns_2005() {
    let result = EventClassSelector::select("crowdstrike", "incident");
    assert!(
        result.is_ok(),
        "crowdstrike/incident must return Ok (BC-2.02.012)"
    );
    assert_eq!(
        result.unwrap(),
        CLASS_UID_INCIDENT_FINDING,
        "crowdstrike/incident must map to Incident Finding (2005) (BC-2.02.012)"
    );
}

/// BC-2.02.012: Cyberint alert → Detection Finding (class_uid 2004).
#[test]
fn test_BC_2_02_012_cyberint_alert_returns_2004() {
    let result = EventClassSelector::select("cyberint", "alert");
    assert!(
        result.is_ok(),
        "cyberint/alert must return Ok (BC-2.02.012)"
    );
    assert_eq!(
        result.unwrap(),
        CLASS_UID_DETECTION_FINDING,
        "cyberint/alert must map to Detection Finding (2004) (BC-2.02.012)"
    );
}

/// BC-2.02.012 / AC-7: Claroty alert → Detection Finding (class_uid 2004).
#[test]
fn test_BC_2_02_012_claroty_alert_returns_2004() {
    let result = EventClassSelector::select("claroty", "alert");
    assert!(
        result.is_ok(),
        "claroty/alert must return Ok (AC-7, BC-2.02.012)"
    );
    assert_eq!(
        result.unwrap(),
        CLASS_UID_DETECTION_FINDING,
        "claroty/alert must map to Detection Finding (2004) (AC-7, BC-2.02.012)"
    );
}

/// BC-2.02.012 / TV-BC-2.02.012-002: Claroty device → Device Inventory Info (5001).
#[test]
fn test_BC_2_02_012_claroty_device_returns_5001() {
    let result = EventClassSelector::select("claroty", "device");
    assert!(
        result.is_ok(),
        "claroty/device must return Ok (BC-2.02.012)"
    );
    assert_eq!(
        result.unwrap(),
        CLASS_UID_DEVICE_INVENTORY_INFO,
        "claroty/device must map to Device Inventory Info (5001) (TV-BC-2.02.012-002)"
    );
}

/// BC-2.02.012 / TV-BC-2.02.012-003: Claroty vulnerability → Vulnerability Finding (2002).
#[test]
fn test_BC_2_02_012_claroty_vulnerability_returns_2002() {
    let result = EventClassSelector::select("claroty", "vulnerability");
    assert!(
        result.is_ok(),
        "claroty/vulnerability must return Ok (BC-2.02.012)"
    );
    assert_eq!(
        result.unwrap(),
        CLASS_UID_VULNERABILITY_FINDING,
        "claroty/vulnerability must map to Vulnerability Finding (2002) (TV-BC-2.02.012-003)"
    );
}

/// BC-2.02.012: Armis device → Device Inventory Info (5001).
#[test]
fn test_BC_2_02_012_armis_device_returns_5001() {
    let result = EventClassSelector::select("armis", "device");
    assert!(result.is_ok(), "armis/device must return Ok (BC-2.02.012)");
    assert_eq!(
        result.unwrap(),
        CLASS_UID_DEVICE_INVENTORY_INFO,
        "armis/device must map to Device Inventory Info (5001) (BC-2.02.012)"
    );
}

/// BC-2.02.012: Armis alert → Detection Finding (2004).
#[test]
fn test_BC_2_02_012_armis_alert_returns_2004() {
    let result = EventClassSelector::select("armis", "alert");
    assert!(result.is_ok(), "armis/alert must return Ok (BC-2.02.012)");
    assert_eq!(
        result.unwrap(),
        CLASS_UID_DETECTION_FINDING,
        "armis/alert must map to Detection Finding (2004) (BC-2.02.012)"
    );
}

/// BC-2.02.012 / TV-BC-2.02.012-004: Armis audit_log → Audit Activity (3001).
#[test]
fn test_BC_2_02_012_armis_audit_log_returns_3001() {
    let result = EventClassSelector::select("armis", "audit_log");
    assert!(
        result.is_ok(),
        "armis/audit_log must return Ok (BC-2.02.012)"
    );
    assert_eq!(
        result.unwrap(),
        CLASS_UID_AUDIT_ACTIVITY,
        "armis/audit_log must map to Audit Activity (3001) (TV-BC-2.02.012-004)"
    );
}

/// BC-2.02.012 / TV-BC-2.02.012-004 (claroty): Claroty audit_log → Audit Activity (3001).
#[test]
fn test_BC_2_02_012_claroty_audit_log_returns_3001() {
    let result = EventClassSelector::select("claroty", "audit_log");
    assert!(
        result.is_ok(),
        "claroty/audit_log must return Ok (BC-2.02.012)"
    );
    assert_eq!(
        result.unwrap(),
        CLASS_UID_AUDIT_ACTIVITY,
        "claroty/audit_log must map to Audit Activity (3001) (BC-2.02.012)"
    );
}

/// BC-2.02.012 / AC-8 / TV-BC-2.02.012-006: completely unknown sensor+record_type
/// returns `Err(OcsfUnknownEventClass)`.
#[test]
fn test_BC_2_02_012_unknown_pair_returns_err() {
    let result = EventClassSelector::select("vendor_x", "unknown_type");
    assert!(
        result.is_err(),
        "vendor_x/unknown_type must return Err (AC-8, BC-2.02.012)"
    );

    let err = result.unwrap_err();
    match &err {
        PrismError::OcsfUnknownEventClass {
            sensor,
            record_type,
        } => {
            assert_eq!(sensor, "vendor_x");
            assert_eq!(record_type, "unknown_type");
        }
        other => panic!(
            "Expected OcsfUnknownEventClass, got {:?} (AC-8, BC-2.02.012)",
            other
        ),
    }
}

/// BC-2.02.012 invariant: deprecated class_uid 2001 (Security Finding) MUST NOT appear
/// in any mapping.
///
/// This test iterates all known sensor/record_type combinations and asserts none
/// returns class_uid 2001. (AC-6 note: "NOT deprecated 2001", BC-2.02.012)
#[test]
fn test_BC_2_02_012_invariant_no_deprecated_2001_in_any_mapping() {
    const DEPRECATED_SECURITY_FINDING: u32 = 2001;

    let test_cases = [
        ("crowdstrike", "detection"),
        ("crowdstrike", "incident"),
        ("cyberint", "alert"),
        ("claroty", "alert"),
        ("claroty", "asset"),
        ("claroty", "device"),
        ("claroty", "vulnerability"),
        ("claroty", "audit_log"),
        ("armis", "device"),
        ("armis", "alert"),
        ("armis", "audit_log"),
    ];

    for (sensor, record_type) in &test_cases {
        let result = EventClassSelector::select(sensor, record_type);
        if let Ok(class_uid) = result {
            assert_ne!(
                class_uid, DEPRECATED_SECURITY_FINDING,
                "{sensor}/{record_type} must NOT map to deprecated Security Finding \
                 (2001) — deprecated since OCSF v1.1.0 (BC-2.02.012 invariant)"
            );
        }
    }
}

/// BC-2.02.012 invariant: select() is deterministic — same inputs always yield same output.
#[test]
fn test_BC_2_02_012_invariant_select_is_deterministic() {
    let first = EventClassSelector::select("crowdstrike", "detection");
    let second = EventClassSelector::select("crowdstrike", "detection");

    // Both should be Ok with the same class_uid — PrismError doesn't impl PartialEq
    // so we compare the Ok values directly.
    assert!(
        first.is_ok(),
        "first call to select() must return Ok (BC-2.02.012)"
    );
    assert!(
        second.is_ok(),
        "second call to select() must return Ok (BC-2.02.012)"
    );
    assert_eq!(
        first.unwrap(),
        second.unwrap(),
        "EventClassSelector::select() must be deterministic — same value on every call \
         (BC-2.02.012 invariant)"
    );
}

/// BC-2.02.012: select() with empty sensor string returns Err.
#[test]
fn test_BC_2_02_012_rejects_empty_sensor() {
    let result = EventClassSelector::select("", "detection");
    assert!(
        result.is_err(),
        "EventClassSelector::select() must return Err for empty sensor (BC-2.02.012)"
    );
}

/// BC-2.02.012: select() with empty record_type string returns Err.
#[test]
fn test_BC_2_02_012_rejects_empty_record_type() {
    let result = EventClassSelector::select("crowdstrike", "");
    assert!(
        result.is_err(),
        "EventClassSelector::select() must return Err for empty record_type (BC-2.02.012)"
    );
}
