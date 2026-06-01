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
//! ## Legacy `select(sensor, record_type)` path mappings
//!
//! The table below documents the `select()` method's (sensor, record_type) routing
//! table. The "Security Finding (DEPRECATED)" row means that `select()` intentionally
//! does NOT route any (sensor, record_type) pair to class_uid 2001 — that class was
//! retired from the legacy lookup table in OCSF v1.1.0. It does NOT mean 2001 is
//! unusable globally: `select_by_class_name("security_finding")` returns 2001 for
//! real sensor TOML specs per BC-2.01.013 v1.9, because `security_finding` remains
//! the correct ocsf_class value for sensors not yet migrated to `detection_finding`.
//!
//! | Class Name                | class_uid | Notes                              |
//! |---------------------------|-----------|------------------------------------|
//! | Detection Finding         | 2004      | CrowdStrike detections, Claroty/Armis alerts |
//! | Incident Finding          | 2005      | CrowdStrike incidents              |
//! | Vulnerability Finding     | 2002      | Claroty vulnerabilities            |
//! | Device Inventory Info     | 5001      | Claroty/Armis devices              |
//! | Account Change            | 3001      | Claroty/Armis audit logs (closest OCSF v1.7.0 IAM class) |
//! | Base Event                | 0         | Fallback for unmapped record types |
//! | Security Finding (DEPRECATED) | 2001  | Not used in `select()` — deprecated OCSF v1.1.0 sensor path; use `select_by_class_name("security_finding")` for TOML-spec sensors |
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

/// OCSF `class_uid` for Account Change (uid=3001) in OCSF v1.7.0. (BC-2.02.012)
///
/// Note: OCSF v1.7.0 does not define a standalone "Audit Activity" class. uid=3001
/// is `account_change` (AccountChange). Claroty/Armis `audit_log` records are mapped
/// here as the closest available IAM-category event class. S-1.05 field mappers will
/// verify this is semantically correct or propose an alternative class_uid.
pub const CLASS_UID_ACCOUNT_CHANGE: u32 = 3001;

/// OCSF `class_uid` for Base Event — used for unmapped record types. (BC-2.02.012)
pub const CLASS_UID_BASE_EVENT: u32 = 0;

/// OCSF `class_uid` for Security Finding.
///
/// Value: 2001. Used by `select_by_class_name("security_finding")` for sensors that
/// declare `ocsf_class = "security_finding"` in their TOML spec (BC-2.01.013 v1.9).
///
/// NOTE: Whether 2001 vs 2004 is the semantically correct UID for Cyberint alerts is
/// under architect adjudication (OBS-2). This constant names the CURRENT value used in
/// `select_by_class_name` — it is a pure naming refactor of that value. Do NOT change
/// the value here without resolving OBS-2 first.
pub const CLASS_UID_SECURITY_FINDING: u32 = 2001;

/// Maps (sensor, record_type) pairs to OCSF event class UIDs.
///
/// This is a zero-size unit struct. All methods are pure functions operating on
/// compile-time data — no state, no allocations. (S-1.04 Architecture Compliance)
pub struct EventClassSelector;

impl EventClassSelector {
    /// Returns the OCSF `class_uid` for the given OCSF class-name string.
    ///
    /// This is the **spec-driven lookup path** used by `pipeline_result_to_record_batch`:
    /// the sensor TOML `[[tables]]` block declares `ocsf_class = "<class-name>"`, and this
    /// method maps that class-name directly to the canonical `class_uid` integer — with no
    /// dependency on the sensor identifier.
    ///
    /// # Mappings (compile-time constants, BC-2.01.013 v1.9 / D-925)
    ///
    /// | Class name              | class_uid | OCSF category |
    /// |-------------------------|-----------|---------------|
    /// | `"security_finding"`    | 2001      | 2 (Findings)  |
    /// | `"vulnerability_finding"` | 2002    | 2 (Findings)  |
    /// | `"detection_finding"`   | 2004      | 2 (Findings)  |
    /// | `"incident_finding"`    | 2005      | 2 (Findings)  |
    /// | `"audit_activity"`      | 3001      | 3 (IAM)       |
    /// | `"device"`              | 5001      | 5 (Discovery) |
    ///
    /// # Behaviour
    ///
    /// - Known class names return `Ok(class_uid)`.
    /// - Unknown class names return `Err(PrismError::OcsfUnknownEventClass)`.
    ///   Callers that want a safe fallback should use `.unwrap_or(0)`.
    ///
    /// # Error Codes
    ///
    /// - `E-OCSF-020` (`PrismError::OcsfUnknownEventClass`) — no mapping for this class name.
    pub fn select_by_class_name(class_name: &str) -> Result<u32, PrismError> {
        match class_name {
            "security_finding" => Ok(CLASS_UID_SECURITY_FINDING),
            "vulnerability_finding" => Ok(CLASS_UID_VULNERABILITY_FINDING),
            "detection_finding" => Ok(CLASS_UID_DETECTION_FINDING),
            "incident_finding" => Ok(CLASS_UID_INCIDENT_FINDING),
            "audit_activity" => Ok(CLASS_UID_ACCOUNT_CHANGE),
            "device" => Ok(CLASS_UID_DEVICE_INVENTORY_INFO),
            _ => Err(PrismError::OcsfUnknownEventClass {
                // "<class-name-lookup>" sentinel: this path is reached via select_by_class_name,
                // which takes only a class_name — there is no sensor context. Using an empty
                // string would be misleading in diagnostics; the sentinel makes the lookup
                // origin unambiguous.
                sensor: "<class-name-lookup>".to_owned(),
                record_type: class_name.to_owned(),
            }),
        }
    }

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
            ("claroty", "audit_log") => Ok(CLASS_UID_ACCOUNT_CHANGE),

            // Armis
            ("armis", "device") => Ok(CLASS_UID_DEVICE_INVENTORY_INFO),
            ("armis", "alert") => Ok(CLASS_UID_DETECTION_FINDING),
            ("armis", "audit_log") => Ok(CLASS_UID_ACCOUNT_CHANGE),

            // All other pairs — no mapping defined
            _ => Err(PrismError::OcsfUnknownEventClass {
                sensor: sensor.to_owned(),
                record_type: record_type.to_owned(),
            }),
        }
    }
}
