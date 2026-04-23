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
        pool.get_message_by_name(&format!("ocsf.v1_x.{class_uid}"))
            .ok_or(PrismError::OcsfDescriptorNotFound { class_uid })
    }
}

impl Default for OcsfNormalizer {
    fn default() -> Self {
        Self::new()
    }
}
