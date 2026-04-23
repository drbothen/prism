//! `OcsfEvent` — thin wrapper around `DynamicMessage` that carries Prism metadata.
//!
//! This type is used by the alias resolver (BC-2.02.008) as the container on which
//! four-tier field resolution operates. The Prism metadata fields (tier 1) live
//! directly on this struct, while the OCSF proto fields (tier 2) and raw_extensions
//! (tier 3) live inside `message` and `raw_extensions` respectively.

use prost_reflect::DynamicMessage;
use serde_json::Value as JsonValue;

/// Prism's wrapper around a normalized OCSF `DynamicMessage`.
///
/// Carries the mandatory Prism metadata fields (tier 1 of alias resolution) alongside
/// the populated OCSF protobuf message (tier 2) and the unmapped field blob (tier 3).
///
/// (BC-2.02.008)
#[derive(Debug)]
pub struct OcsfEvent {
    /// The normalized OCSF `DynamicMessage`.
    pub message: DynamicMessage,

    /// Prism metadata — tier 1 fields for alias resolution. (BC-2.02.008 postcondition 1)
    pub source_sensor: String,
    pub source_record_type: String,
    pub client_id: String,

    /// Vendor-specific fields that had no OCSF mapping. (BC-2.02.007)
    pub raw_extensions: serde_json::Map<String, JsonValue>,
}

impl OcsfEvent {
    /// Constructs a new `OcsfEvent`.
    ///
    /// # Parameters
    ///
    /// - `message`: the normalized OCSF `DynamicMessage` (tier 2 fields)
    /// - `source_sensor`: the sensor that produced this event (tier 1)
    /// - `source_record_type`: the vendor record type (tier 1)
    /// - `client_id`: the Prism client/tenant identifier (tier 1)
    /// - `raw_extensions`: unmapped vendor fields preserved as JSON (tier 3)
    pub fn new(
        message: DynamicMessage,
        source_sensor: impl Into<String>,
        source_record_type: impl Into<String>,
        client_id: impl Into<String>,
        raw_extensions: serde_json::Map<String, JsonValue>,
    ) -> Self {
        OcsfEvent {
            message,
            source_sensor: source_sensor.into(),
            source_record_type: source_record_type.into(),
            client_id: client_id.into(),
            raw_extensions,
        }
    }
}
