//! OCSF normalizer — dispatches to per-sensor `SensorMapper` implementations.
//!
//! BC-2.02.002: `OcsfNormalizer::normalize()` creates a `DynamicMessage` wrapping the
//! target OCSF event class protobuf descriptor, then delegates field population to the
//! sensor-specific mapper (S-1.05). The normalizer dispatches via `SensorMapper` trait,
//! never via `match sensor {}`. (S-1.05 Architecture Compliance Rules)
//!
//! # Panic Safety (VP-022)
//!
//! `normalize()` MUST NOT panic. All errors returned via `Result`.

use prism_core::PrismError;
use prost_reflect::{DynamicMessage, MessageDescriptor};
use serde_json::Value;

use crate::class_selector::EventClassSelector;
use crate::mappers::SensorMapper;
use crate::pool::OcsfDescriptors;

/// OCSF normalizer — dispatches to per-sensor `SensorMapper` implementations.
///
/// # Thread Safety
///
/// `OcsfNormalizer` is `Send + Sync` — holds no mutable state after construction.
pub struct OcsfNormalizer {
    /// Registered sensor mappers, dispatched by `sensor_id()`. (S-1.05 Task 1)
    mappers: Vec<Box<dyn SensorMapper>>,
}

// Safety: OcsfNormalizer holds a Vec of trait objects that are themselves Send + Sync.
// The Vec is never mutated after construction.
unsafe impl Send for OcsfNormalizer {}
unsafe impl Sync for OcsfNormalizer {}

impl OcsfNormalizer {
    /// Creates a new `OcsfNormalizer` with no registered mappers.
    pub fn new() -> Self {
        OcsfNormalizer {
            mappers: Vec::new(),
        }
    }

    /// Creates an `OcsfNormalizer` pre-loaded with the provided sensor mappers.
    ///
    /// The normalizer dispatches to mappers by matching `sensor_id()` against the
    /// incoming record's sensor label. (S-1.05 Task 1, Architecture Compliance Rules)
    pub fn with_mappers(mappers: Vec<Box<dyn SensorMapper>>) -> Self {
        OcsfNormalizer { mappers }
    }

    /// Normalizes a raw sensor record to an OCSF `DynamicMessage`, dispatching to the
    /// appropriate registered `SensorMapper` for field population. (BC-2.02.002, S-1.05)
    ///
    /// # Steps
    ///
    /// 1. Call `EventClassSelector::select(sensor, record_type)` to get `class_uid`.
    /// 2. Look up the `MessageDescriptor` from the pool for that class.
    /// 3. Create an empty `DynamicMessage`.
    /// 4. Find the `SensorMapper` whose `sensor_id()` matches `sensor` and whose
    ///    `record_types()` includes `record_type`.
    /// 5. Call `mapper.map(record_type, raw, &mut msg, &mut extensions)`.
    /// 6. Return the populated `DynamicMessage` + source_record_id.
    ///
    /// # Errors
    ///
    /// - `PrismError::OcsfUnknownEventClass` — no class mapping for sensor+record_type.
    /// - `PrismError::OcsfDescriptorNotFound` — class_uid not in pool.
    /// - `PrismError::OcsfNormalizationFailed` — normalization failure or no mapper found.
    /// - `PrismError::OcsfUnknownRecordType` — mapper found but doesn't handle record_type.
    ///
    /// # Panics
    ///
    /// Never. (VP-022)
    pub fn normalize_with_mappers(
        &self,
        sensor: &str,
        record_type: &str,
        raw: Value,
    ) -> Result<(DynamicMessage, String), PrismError> {
        let class_uid = EventClassSelector::select(sensor, record_type)?;
        let descriptor = Self::descriptor_for_class_uid(class_uid)?;
        let mut msg = DynamicMessage::new(descriptor);
        let mut extensions = serde_json::Map::new();

        // Find the mapper for this sensor (dispatches via SensorMapper trait, not match).
        let mapper = self
            .mappers
            .iter()
            .find(|m| m.sensor_id() == sensor)
            .ok_or_else(|| PrismError::OcsfNormalizationFailed {
                source_id: format!("<{sensor}>"),
                reason: format!("no mapper registered for sensor '{sensor}'"),
            })?;

        let source_id = mapper.map(record_type, &raw, &mut msg, &mut extensions)?;
        Ok((msg, source_id))
    }

    /// Legacy entry point retained from S-1.04 (no mapper dispatch).
    ///
    /// Looks up the event class descriptor for the given sensor + record_type pair and
    /// returns an empty `DynamicMessage`. Field population is deferred to `normalize_with_mappers`.
    pub fn normalize(
        &self,
        sensor: &str,
        record_type: &str,
        _raw: Value,
    ) -> Result<DynamicMessage, PrismError> {
        let class_uid = EventClassSelector::select(sensor, record_type)?;
        let descriptor = Self::descriptor_for_class_uid(class_uid)?;
        let message = DynamicMessage::new(descriptor);
        Ok(message)
    }

    fn descriptor_for_class_uid(class_uid: u32) -> Result<MessageDescriptor, PrismError> {
        let pool = OcsfDescriptors::get();

        let msg_name = ocsf_class_uid_to_message_name(class_uid)
            .ok_or(PrismError::OcsfDescriptorNotFound { class_uid })?;

        pool.get_message_by_name(msg_name)
            .ok_or(PrismError::OcsfDescriptorNotFound { class_uid })
    }
}

impl Default for OcsfNormalizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Maps an OCSF `class_uid` to the fully-qualified protobuf message name in the
/// descriptor pool.
///
/// The naming convention is `ocsf.v1_7_0.events.{category}.{PascalCaseName}`,
/// where `{category}` is the OCSF event category (e.g., `findings`, `iam`, `discovery`)
/// and `{PascalCaseName}` is the class name converted from snake_case to PascalCase.
///
/// These mappings are verified against OCSF v1.7.0 (the pinned version). Returns
/// `None` for `class_uid` values not present in the schema.
fn ocsf_class_uid_to_message_name(class_uid: u32) -> Option<&'static str> {
    match class_uid {
        // ── System (category: system) ──────────────────────────────────────
        1001 => Some("ocsf.v1_7_0.events.system.FileActivity"),
        1002 => Some("ocsf.v1_7_0.events.system.KernelExtensionActivity"),
        1003 => Some("ocsf.v1_7_0.events.system.KernelActivity"),
        1004 => Some("ocsf.v1_7_0.events.system.MemoryActivity"),
        1005 => Some("ocsf.v1_7_0.events.system.ModuleActivity"),
        1006 => Some("ocsf.v1_7_0.events.system.ScheduledJobActivity"),
        1007 => Some("ocsf.v1_7_0.events.system.ProcessActivity"),
        1008 => Some("ocsf.v1_7_0.events.system.EventLogActvity"), // typo in OCSF schema
        1009 => Some("ocsf.v1_7_0.events.system.ScriptActivity"),
        1010 => Some("ocsf.v1_7_0.events.system.PeripheralActivity"),
        // Windows extensions (uid 201xxx, category: system)
        201001 => Some("ocsf.v1_7_0.events.system.RegistryKeyActivity"),
        201002 => Some("ocsf.v1_7_0.events.system.RegistryValueActivity"),
        201003 => Some("ocsf.v1_7_0.events.system.WindowsResourceActivity"),
        201004 => Some("ocsf.v1_7_0.events.system.WindowsServiceActivity"),

        // ── Findings (category: findings) ─────────────────────────────────
        2001 => Some("ocsf.v1_7_0.events.findings.SecurityFinding"), // deprecated OCSF v1.1.0
        2002 => Some("ocsf.v1_7_0.events.findings.VulnerabilityFinding"),
        2003 => Some("ocsf.v1_7_0.events.findings.ComplianceFinding"),
        2004 => Some("ocsf.v1_7_0.events.findings.DetectionFinding"),
        2005 => Some("ocsf.v1_7_0.events.findings.IncidentFinding"),
        2006 => Some("ocsf.v1_7_0.events.findings.DataSecurityFinding"),
        2007 => Some("ocsf.v1_7_0.events.findings.ApplicationSecurityPostureFinding"),
        2008 => Some("ocsf.v1_7_0.events.findings.IamAnalysisFinding"),

        // ── IAM (category: iam) ───────────────────────────────────────────
        3001 => Some("ocsf.v1_7_0.events.iam.AccountChange"),
        3002 => Some("ocsf.v1_7_0.events.iam.Authentication"),
        3003 => Some("ocsf.v1_7_0.events.iam.AuthorizeSession"),
        3004 => Some("ocsf.v1_7_0.events.iam.EntityManagement"),
        3005 => Some("ocsf.v1_7_0.events.iam.UserAccess"),
        3006 => Some("ocsf.v1_7_0.events.iam.GroupManagement"),

        // ── Network (category: network) ───────────────────────────────────
        4001 => Some("ocsf.v1_7_0.events.network.NetworkActivity"),
        4002 => Some("ocsf.v1_7_0.events.network.HttpActivity"),
        4003 => Some("ocsf.v1_7_0.events.network.DnsActivity"),
        4004 => Some("ocsf.v1_7_0.events.network.DhcpActivity"),
        4005 => Some("ocsf.v1_7_0.events.network.RdpActivity"),
        4006 => Some("ocsf.v1_7_0.events.network.SmbActivity"),
        4007 => Some("ocsf.v1_7_0.events.network.SshActivity"),
        4008 => Some("ocsf.v1_7_0.events.network.FtpActivity"),
        4009 => Some("ocsf.v1_7_0.events.network.EmailActivity"),
        4010 => Some("ocsf.v1_7_0.events.network.NetworkFileActivity"),
        4011 => Some("ocsf.v1_7_0.events.network.EmailFileActivity"),
        4012 => Some("ocsf.v1_7_0.events.network.EmailUrlActivity"),
        4013 => Some("ocsf.v1_7_0.events.network.NtpActivity"),
        4014 => Some("ocsf.v1_7_0.events.network.TunnelActivity"),

        // ── Discovery (category: discovery) ───────────────────────────────
        5001 => Some("ocsf.v1_7_0.events.discovery.InventoryInfo"),
        5002 => Some("ocsf.v1_7_0.events.discovery.ConfigState"),
        5003 => Some("ocsf.v1_7_0.events.discovery.UserInventory"),
        5004 => Some("ocsf.v1_7_0.events.discovery.PatchState"),
        5006 => Some("ocsf.v1_7_0.events.discovery.KernelObjectQuery"),
        5007 => Some("ocsf.v1_7_0.events.discovery.FileQuery"),
        5008 => Some("ocsf.v1_7_0.events.discovery.FolderQuery"),
        5009 => Some("ocsf.v1_7_0.events.discovery.AdminGroupQuery"),
        5010 => Some("ocsf.v1_7_0.events.discovery.JobQuery"),
        5011 => Some("ocsf.v1_7_0.events.discovery.ModuleQuery"),
        5012 => Some("ocsf.v1_7_0.events.discovery.NetworkConnectionQuery"),
        5013 => Some("ocsf.v1_7_0.events.discovery.NetworksQuery"),
        5014 => Some("ocsf.v1_7_0.events.discovery.PeripheralDeviceQuery"),
        5015 => Some("ocsf.v1_7_0.events.discovery.ProcessQuery"),
        5016 => Some("ocsf.v1_7_0.events.discovery.ServiceQuery"),
        5017 => Some("ocsf.v1_7_0.events.discovery.SessionQuery"),
        5018 => Some("ocsf.v1_7_0.events.discovery.UserQuery"),
        5019 => Some("ocsf.v1_7_0.events.discovery.DeviceConfigStateChange"),
        5020 => Some("ocsf.v1_7_0.events.discovery.SoftwareInfo"),
        5021 => Some("ocsf.v1_7_0.events.discovery.OsintInventoryInfo"),
        5022 => Some("ocsf.v1_7_0.events.discovery.StartupItemQuery"),
        5023 => Some("ocsf.v1_7_0.events.discovery.CloudResourcesInventoryInfo"),
        5040 => Some("ocsf.v1_7_0.events.discovery.EvidenceInfo"),
        // Discovery Windows extensions
        205004 => Some("ocsf.v1_7_0.events.discovery.RegistryKeyQuery"),
        205005 => Some("ocsf.v1_7_0.events.discovery.RegistryValueQuery"),
        205019 => Some("ocsf.v1_7_0.events.discovery.PrefetchQuery"),

        // ── Application (category: application) ───────────────────────────
        6001 => Some("ocsf.v1_7_0.events.application.WebResourcesActivity"),
        6002 => Some("ocsf.v1_7_0.events.application.ApplicationLifecycle"),
        6003 => Some("ocsf.v1_7_0.events.application.ApiActivity"),
        6004 => Some("ocsf.v1_7_0.events.application.WebResourceAccessActivity"),
        6005 => Some("ocsf.v1_7_0.events.application.DatastoreActivity"),
        6006 => Some("ocsf.v1_7_0.events.application.FileHosting"),
        6007 => Some("ocsf.v1_7_0.events.application.ScanActivity"),
        6008 => Some("ocsf.v1_7_0.events.application.ApplicationError"),

        // ── Remediation (category: remediation) ───────────────────────────
        7001 => Some("ocsf.v1_7_0.events.remediation.RemediationActivity"),
        7002 => Some("ocsf.v1_7_0.events.remediation.FileRemediationActivity"),
        7003 => Some("ocsf.v1_7_0.events.remediation.ProcessRemediationActivity"),
        7004 => Some("ocsf.v1_7_0.events.remediation.NetworkRemediationActivity"),

        // ── Unmanned Systems (category: unmanned_systems) ─────────────────
        8001 => Some("ocsf.v1_7_0.events.unmanned_systems.DroneFlightsActivity"),
        8002 => Some("ocsf.v1_7_0.events.unmanned_systems.AirborneBroadcastActivity"),

        // Unknown class_uid — not in OCSF v1.7.0 schema.
        _ => None,
    }
}
