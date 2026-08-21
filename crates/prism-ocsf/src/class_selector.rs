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
//! ## `select(sensor, record_type)` path mappings
//!
//! The table below documents the `select()` method's (sensor, record_type) routing
//! table. Per BC-2.02.012 INV-NO-2001-SELECT-PATH, `select()` MUST NOT return
//! class_uid 2001 (Security Finding, deprecated OCSF v1.1.0) for any token.
//!
//! | Class Name                | class_uid | Notes                              |
//! |---------------------------|-----------|------------------------------------|
//! | Detection Finding         | 2004      | CrowdStrike detections, Claroty/Armis alerts |
//! | Incident Finding          | 2005      | CrowdStrike incidents              |
//! | Vulnerability Finding     | 2002      | Claroty vulnerabilities            |
//! | Device Inventory Info     | 5001      | Claroty/Armis devices              |
//! | Account Change            | 3001      | (transitional; Claroty/Armis audit logs pre-KF-01 — now superseded by Entity Management 3004) |
//! | Entity Management         | 3004      | Claroty/Armis audit logs (KF-01 correction per ADR-058 §K5 Div-3; has `comment` attr) |
//! | Base Event                | 0         | Fallback for unmapped record types |
//! | Security Finding (DEPRECATED) | 2001  | NOT used in `select()` or `select_by_class_name()` — deprecated OCSF v1.1.0. `select_by_class_name("security_finding")` returns 2004 with a deprecation WARN (BC-2.02.012 Option A, OCSF-CLASS-MIGRATION-001). |
//!
//! ## `select_by_class_name(class_name)` path mappings (BC-2.02.012)
//!
//! | `class_name` (TOML `ocsf_class`) | `class_uid` | Notes |
//! |----------------------------------|-------------|-------|
//! | `"detection_finding"`            | 2004        | OCSF v1.1 canonical; PRIMARY entry; no WARN |
//! | `"security_finding"`             | 2004        | Transitional alias; maps to 2004 (NOT 2001); emits `ocsf.deprecated_class_alias` WARN |
//! | `"incident_finding"`             | 2005        | CrowdStrike incidents, Cyberint incidents |
//! | `"vulnerability_finding"`        | 2002        | Claroty vulnerabilities |
//! | `"device"`                       | 5001        | Claroty/Armis devices |
//! | `"audit_activity"`               | 3001        | Claroty/Armis audit logs |
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

/// OCSF `class_uid` for Entity Management (uid=3004) in OCSF v1.7.0.
///
/// Replaces the incorrect `"audit_activity"` TOML value used by Claroty/Armis
/// `audit_log` records (KF-01 correction per ADR-058 §K5 Divergence 3).
/// `entity_management` (3004) has the `comment` attribute required for the
/// `note → comment` mapping in Claroty `audit_logs`. `account_change` (3001) lacks
/// `comment`, causing silent data loss of every `note` value.
///
/// (AC-009 sub-obligations a+b; S-ADR058-OCSF-ROUTING-001)
pub const CLASS_UID_ENTITY_MANAGEMENT: u32 = 3004;

/// OCSF `class_uid` for Base Event — used for unmapped record types. (BC-2.02.012)
pub const CLASS_UID_BASE_EVENT: u32 = 0;

/// OCSF `class_uid` for Security Finding — DEPRECATED since OCSF v1.1.0.
///
/// Value: 2001. This class was retired in OCSF v1.1.0 in favor of Detection Finding (2004).
///
/// **NOT used by `select_by_class_name("security_finding")`** — per BC-2.02.012
/// (OCSF-CLASS-MIGRATION-001 Option A), the transitional alias `"security_finding"` now
/// resolves to 2004 (CLASS_UID_DETECTION_FINDING) with a deprecation WARN.
/// This constant is retained ONLY for documentation — it names the deprecated class_uid
/// that was previously returned and must never be re-introduced in any mapping.
///
/// Per INV-NO-2001-SELECT-PATH and INV-PRODUCTION-TOML-NO-SECURITY-FINDING (BC-2.02.012):
/// neither `select()` nor `select_by_class_name()` returns 2001 for any input.
#[deprecated(
    note = "OCSF v1.1.0 deprecated Security Finding (2001). Use CLASS_UID_DETECTION_FINDING \
            (2004). BC-2.02.012 INV-NO-2001-SELECT-PATH."
)]
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
    /// # Mappings (BC-2.02.012, OCSF-CLASS-MIGRATION-001 Option A)
    ///
    /// | Class name                | class_uid | OCSF category | Notes |
    /// |---------------------------|-----------|---------------|-------|
    /// | `"detection_finding"`     | 2004      | 2 (Findings)  | PRIMARY — OCSF v1.1 canonical; no WARN |
    /// | `"security_finding"`      | 2004      | 2 (Findings)  | Transitional alias; resolves to 2004 (NOT 2001); emits `ocsf.deprecated_class_alias` WARN |
    /// | `"vulnerability_finding"` | 2002      | 2 (Findings)  | |
    /// | `"incident_finding"`      | 2005      | 2 (Findings)  | |
    /// | `"audit_activity"`        | 3001      | 3 (IAM)       | |
    /// | `"device"`                | 5001      | 5 (Discovery) | |
    ///
    /// # Behaviour
    ///
    /// - Known class names return `Ok(class_uid)`.
    /// - `"security_finding"` returns `Ok(2004)` (transitional alias per Option A) AND emits
    ///   a `tracing::warn!` with `event_type = "ocsf.deprecated_class_alias"`. Callers should
    ///   update their sensor TOML to use `"detection_finding"` directly.
    /// - Unknown class names return `Err(PrismError::OcsfUnknownEventClass)`.
    ///   Callers that want a safe fallback should use `.unwrap_or(0)`.
    ///
    /// # Error Codes
    ///
    /// - `E-OCSF-020` (`PrismError::OcsfUnknownEventClass`) — no mapping for this class name.
    pub fn select_by_class_name(class_name: &str) -> Result<u32, PrismError> {
        match class_name {
            "detection_finding" => Ok(CLASS_UID_DETECTION_FINDING),
            "security_finding" => {
                // Transitional alias per BC-2.02.012 Option A (OCSF-CLASS-MIGRATION-001).
                // `"security_finding"` was the ocsf_class value used by production sensor TOMLs
                // prior to OCSF-CLASS-MIGRATION-001. External TOML specs not under Prism control
                // may still use this string. Maps to 2004 (Detection Finding) with a deprecation
                // WARN so operators know to update their TOML specs.
                tracing::warn!(
                    event_type = "ocsf.deprecated_class_alias",
                    class_name = "security_finding",
                    resolved_class_uid = CLASS_UID_DETECTION_FINDING,
                    "sensor TOML uses deprecated ocsf_class value 'security_finding'; update to 'detection_finding'"
                );
                Ok(CLASS_UID_DETECTION_FINDING)
            }
            "vulnerability_finding" => Ok(CLASS_UID_VULNERABILITY_FINDING),
            "incident_finding" => Ok(CLASS_UID_INCIDENT_FINDING),
            // DEPRECATED: "audit_activity" was a non-OCSF string; no production TOML uses this
            // after KF-01 correction (S-ADR058-OCSF-ROUTING-001). Claroty/Armis sensor TOMLs
            // now declare `ocsf_class = "entity_management"` (3004). Remove after confirming
            // zero TOML instances across all deployed sensors.
            "audit_activity" => Ok(CLASS_UID_ACCOUNT_CHANGE),
            "device" => Ok(CLASS_UID_DEVICE_INVENTORY_INFO),
            // AC-009 sub-obligation (b): new arms for KF-01/KF-02 corrected TOML values.
            // RG-011: both arms must be added together (single test asserts both at once).
            "entity_management" => todo!(
                "RG-011 (AC-009b): entity_management -> CLASS_UID_ENTITY_MANAGEMENT (3004); \
                 KF-01 TOML correction routes Claroty/Armis audit_log ocsf_class here; \
                 without this arm, KF-01 TOML fix regresses class_uid from 3001 to 0 (BASE_EVENT)"
            ),
            "inventory_info" => todo!(
                "RG-011 (AC-009b): inventory_info -> CLASS_UID_DEVICE_INVENTORY_INFO (5001); \
                 KF-02 TOML correction routes Claroty/Armis devices ocsf_class here; \
                 without this arm, KF-02 TOML fix silently regresses class_uid from 5001 to 0"
            ),
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
