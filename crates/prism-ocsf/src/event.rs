//! `OcsfEvent` — thin wrapper around `DynamicMessage` that carries Prism metadata.
//!
//! This type is used by the alias resolver (BC-2.02.008) as the container on which
//! four-tier field resolution operates. The Prism metadata fields (tier 1) live
//! directly on this struct, while the OCSF proto fields (tier 2) and raw_extensions
//! (tier 3) live inside `message` and `raw_extensions` respectively.
//!
//! # Stub Status (S-1.05 Red Gate)
//!
//! Body is `unimplemented!()` — all field accessor logic deferred to implementation.

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
    /// # Stub — body unimplemented.
    pub fn new(
        _message: DynamicMessage,
        _source_sensor: impl Into<String>,
        _source_record_type: impl Into<String>,
        _client_id: impl Into<String>,
        _raw_extensions: serde_json::Map<String, JsonValue>,
    ) -> Self {
        unimplemented!("OcsfEvent::new — S-1.05 stub, implement in S-1.05")
    }
}
