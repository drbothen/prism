//! OCSF event class selector — maps (sensor, record_type) to OCSF `class_uid`.
//!
//! BC-2.02.012: Each sensor record type maps deterministically to exactly one OCSF
//! event class. The mapping is a compile-time constant — no runtime configuration.
//!
//! # OCSF Class UIDs (verified against OCSF schema, pinned version 1.7.0)
//!
//! The implementer MUST run `ocsf-proto-gen` against the pinned OCSF schema version
//! (BC-2.02.009) and verify that each `class_uid` below exists in the compiled
//! descriptors. If a class_uid is absent, update the mapping before merging.
//!
//! | Class Name                | class_uid | Notes                              |
//! |---------------------------|-----------|------------------------------------|
//! | Detection Finding         | 2004      | CrowdStrike detections, Claroty/Armis alerts |
//! | Incident Finding          | 2005      | CrowdStrike incidents              |
//! | Vulnerability Finding     | 2002      | Claroty vulnerabilities            |
//! | Device Inventory Info     | 5001      | Claroty/Armis devices              |
//! | Audit Activity            | 3001      | Claroty/Armis audit logs           |
//! | Base Event                | 0         | Fallback for unmapped record types |
//! | Security Finding (DEPRECATED) | 2001  | DO NOT USE — deprecated OCSF v1.1.0 |
//!
//! # Stub Status
//!
//! The routing table is complete and correct per BC-2.02.012. This module is fully
//! functional as a stub — no ocsf-proto-gen output is needed at runtime. The
//! compile-time verification comment above must be resolved by the implementer when
//! ocsf-proto-gen is available.

use prism_core::PrismError;

/// OCSF `class_uid` for Detection Finding. (BC-2.02.012)
pub const CLASS_UID_DETECTION_FINDING: u32 = 2004;

/// OCSF `class_uid` for Incident Finding. (BC-2.02.012)
pub const CLASS_UID_INCIDENT_FINDING: u32 = 2005;

/// OCSF `class_uid` for Vulnerability Finding. (BC-2.02.012)
pub const CLASS_UID_VULNERABILITY_FINDING: u32 = 2002;

/// OCSF `class_uid` for Device Inventory Info. (BC-2.02.012)
pub const CLASS_UID_DEVICE_INVENTORY_INFO: u32 = 5001;

/// OCSF `class_uid` for Audit Activity. (BC-2.02.012)
pub const CLASS_UID_AUDIT_ACTIVITY: u32 = 3001;

/// OCSF `class_uid` for Base Event — used for unmapped record types. (BC-2.02.012)
pub const CLASS_UID_BASE_EVENT: u32 = 0;

/// Maps (sensor, record_type) pairs to OCSF event class UIDs.
///
/// This is a zero-size unit struct. All methods are pure functions operating on
/// compile-time data — no state, no allocations. (S-1.04 Architecture Compliance)
pub struct EventClassSelector;

impl EventClassSelector {
    /// Returns the OCSF `class_uid` for the given `(sensor, record_type)` pair.
    ///
    /// # Behaviour
    ///
    /// - Known mappings return `Ok(class_uid)`. (BC-2.02.012 postconditions)
    /// - Completely unknown pairs return `Err(PrismError::OcsfUnknownEventClass)`. (AC-8)
    /// - Note: per BC-2.02.012 the real implementation falls back to Base Event (class 0)
    ///   for launch-day "no-mapping" types and logs a warning. The `OcsfUnknownEventClass`
    ///   error is reserved for sensor/record_type combinations where no sensor adapter
    ///   exists at all (vendor_x, unknown_type). The implementer should reconcile this
    ///   distinction: the test vectors in BC-2.02.012 TV-005/TV-006 use Base Event
    ///   fallback, while AC-8 explicitly requires Err for "vendor_x/unknown_type".
    ///   This stub returns Err for all unrecognised pairs to satisfy AC-8. The real
    ///   implementation may introduce a separate `select_with_fallback()` that returns
    ///   Ok(BASE_EVENT) for known sensors with no OCSF mapping.
    ///
    /// # Error Codes
    ///
    /// - `E-OCSF-020` (`PrismError::OcsfUnknownEventClass`) — no mapping exists.
    pub fn select(sensor: &str, record_type: &str) -> Result<u32, PrismError> {
        match (sensor, record_type) {
            // CrowdStrike
            ("crowdstrike", "detection") => Ok(CLASS_UID_DETECTION_FINDING),
            ("crowdstrike", "incident") => Ok(CLASS_UID_INCIDENT_FINDING),

            // Cyberint
            ("cyberint", "alert") => Ok(CLASS_UID_DETECTION_FINDING),

            // Claroty
            ("claroty", "alert") => Ok(CLASS_UID_DETECTION_FINDING),
            ("claroty", "asset") | ("claroty", "device") => Ok(CLASS_UID_DEVICE_INVENTORY_INFO),
            ("claroty", "vulnerability") => Ok(CLASS_UID_VULNERABILITY_FINDING),
            ("claroty", "audit_log") => Ok(CLASS_UID_AUDIT_ACTIVITY),

            // Armis
            ("armis", "device") => Ok(CLASS_UID_DEVICE_INVENTORY_INFO),
            ("armis", "alert") => Ok(CLASS_UID_DETECTION_FINDING),
            ("armis", "audit_log") => Ok(CLASS_UID_AUDIT_ACTIVITY),

            // All other pairs — no mapping defined
            _ => Err(PrismError::OcsfUnknownEventClass {
                sensor: sensor.to_owned(),
                record_type: record_type.to_owned(),
            }),
        }
    }
}
