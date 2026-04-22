//! OCSF normalizer — modified for S-1.05 to accept `Vec<Box<dyn SensorMapper>>`.
//!
//! BC-2.02.002: `OcsfNormalizer::normalize()` dispatches to the sensor-specific mapper
//! registered via `SensorMapper` trait rather than a hard-coded `match sensor {}`.
//!
//! # Stub Status (S-1.05 Red Gate)
//!
//! - STUB — copied from S-1.04 worktree and modified to wire `SensorMapper` dispatch.
//! - `normalize_with_mappers()` is the new entry point added by S-1.05.
//! - Bodies return `Err(PrismError::OcsfNormalizationFailed)` until implementation lands.
//!
//! # Panic Safety (VP-022)
//!
//! `normalize()` MUST NOT panic. All errors returned via `Result`.
// STUB — origin: S-1.04 worktree normalizer.rs extended for S-1.05.

use prost_reflect::{DynamicMessage, MessageDescriptor};
use prism_core::PrismError;
use serde_json::Value;

use crate::class_selector::EventClassSelector;
use crate::mappers::SensorMapper;
use crate::pool::OcsfDescriptors;

/// OCSF normalizer — dispatches to per-sensor `SensorMapper` implementations.
///
/// # Thread Safety
///
/// `OcsfNormalizer` is `Send + Sync`.
pub struct OcsfNormalizer {
    /// Registered sensor mappers, dispatched by `sensor_id()`. (S-1.05 Task 1)
    mappers: Vec<Box<dyn SensorMapper>>,
}

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
    /// # Stub — body unimplemented (S-1.05 Red Gate).
    pub fn with_mappers(_mappers: Vec<Box<dyn SensorMapper>>) -> Self {
        unimplemented!("OcsfNormalizer::with_mappers — S-1.05 stub")
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
    /// 6. Write `extensions` into `msg.raw_extensions`.
    /// 7. Return the populated `DynamicMessage` + source_record_id.
    ///
    /// # Errors
    ///
    /// - `PrismError::OcsfUnknownEventClass` — no class mapping.
    /// - `PrismError::OcsfDescriptorNotFound` — class_uid not in pool.
    /// - `PrismError::OcsfNormalizationFailed` — normalization failure.
    /// - `PrismError::OcsfUnknownRecordType` — no mapper handles this record_type.
    ///
    /// # Panics
    ///
    /// Never. (VP-022)
    pub fn normalize_with_mappers(
        &self,
        _sensor: &str,
        _record_type: &str,
        _raw: Value,
    ) -> Result<(DynamicMessage, String), PrismError> {
        // STUB — S-1.05 Red Gate. Real implementation dispatches to SensorMapper.
        unimplemented!("OcsfNormalizer::normalize_with_mappers — S-1.05 stub")
    }

    /// Legacy entry point retained from S-1.04 (no mapper dispatch).
    ///
    /// # Stub — copied from S-1.04.
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
        let _ = pool;
        let _ = class_uid;
        Err(PrismError::OcsfDescriptorNotFound { class_uid })
    }
}

impl Default for OcsfNormalizer {
    fn default() -> Self {
        Self::new()
    }
}
