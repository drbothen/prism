//! OCSF normalizer — converts raw sensor JSON to a `DynamicMessage`.
//!
//! BC-2.02.002: `OcsfNormalizer::normalize()` creates a `DynamicMessage` wrapping the
//! target OCSF event class protobuf descriptor, then delegates field population to the
//! sensor-specific mapper (provided by S-1.05).
//!
//! # Panic Safety (VP-022)
//!
//! `normalize()` MUST NOT panic. All errors are returned via `Result`. The VP-022 fuzz
//! target in `fuzz/fuzz_targets/normalize_fuzz.rs` enforces this property.
//!
//! # Send + Sync
//!
//! `OcsfNormalizer` is `Send + Sync` — it holds no state other than a reference to the
//! static `DescriptorPool` and is used from the async tokio runtime.
//!
//! # Stub Status
//!
//! Steps 1–3 of `normalize()` are stubbed. Step 4 (field population via sensor-specific
//! mapper) is explicitly deferred to S-1.05. The stub returns
//! `Err(PrismError::OcsfNormalizationFailed)` for all inputs until the real descriptor
//! pool and field mappers are available. Tests exercising `Ok(DynamicMessage)` will fail
//! (Red Gate) until the real implementation lands.

use prost_reflect::{DynamicMessage, MessageDescriptor};
use prism_core::PrismError;
use serde_json::Value;

use crate::class_selector::EventClassSelector;
use crate::pool::OcsfDescriptors;

/// OCSF normalizer.
///
/// Converts raw sensor records (as `serde_json::Value`) into `DynamicMessage` instances
/// conforming to the pinned OCSF protobuf schema.
///
/// # Thread Safety
///
/// `OcsfNormalizer` is `Send + Sync` — holds no mutable state.
pub struct OcsfNormalizer;

// Safety: OcsfNormalizer is a zero-size unit struct with no mutable interior state.
// DescriptorPool is accessed via a `&'static` reference from OnceLock.
unsafe impl Send for OcsfNormalizer {}
unsafe impl Sync for OcsfNormalizer {}

impl OcsfNormalizer {
    /// Creates a new `OcsfNormalizer`.
    ///
    /// Does NOT initialize the descriptor pool — `OcsfDescriptors::get()` handles that
    /// lazily on first call to `normalize()`.
    pub fn new() -> Self {
        OcsfNormalizer
    }

    /// Normalizes a raw sensor record to an OCSF `DynamicMessage`.
    ///
    /// # Steps (BC-2.02.002)
    ///
    /// 1. Call `EventClassSelector::select(sensor, record_type)` to get `class_uid`.
    /// 2. Look up the `MessageDescriptor` from `DescriptorPool` for that class.
    /// 3. Create an empty `DynamicMessage` for the descriptor.
    /// 4. Delegate field population to the sensor-specific mapper (S-1.05 — not yet implemented).
    /// 5. Return the populated `DynamicMessage`.
    ///
    /// # Errors
    ///
    /// - `PrismError::OcsfUnknownEventClass` — no OCSF class mapping for this sensor+record_type.
    /// - `PrismError::OcsfDescriptorNotFound` — class_uid not in the descriptor pool.
    /// - `PrismError::OcsfNormalizationFailed` — any other normalization failure.
    ///
    /// # Panics
    ///
    /// Never. All errors are returned via `Result`. (VP-022)
    pub fn normalize(
        &self,
        sensor: &str,
        record_type: &str,
        _raw: Value,
    ) -> Result<DynamicMessage, PrismError> {
        // Step 1: Resolve class_uid.
        let class_uid = EventClassSelector::select(sensor, record_type)?;

        // Step 2: Look up the MessageDescriptor from the pool.
        let descriptor = Self::descriptor_for_class_uid(class_uid)?;

        // Step 3: Create an empty DynamicMessage.
        let message = DynamicMessage::new(descriptor);

        // Step 4: Field population is deferred to S-1.05 sensor-specific mappers.
        // STUB: No field mapping is performed here. The returned DynamicMessage has
        // no fields set other than the defaults. S-1.05 will replace this with real
        // per-sensor field mappers.

        Ok(message)
    }

    /// Looks up the `MessageDescriptor` for a given OCSF `class_uid` from the
    /// compiled descriptor pool.
    ///
    /// # Errors
    ///
    /// Returns `Err(PrismError::OcsfDescriptorNotFound)` if the pool does not contain
    /// a descriptor for the given `class_uid`. (AC-2, BC-2.02.001, E-OCSF-022)
    fn descriptor_for_class_uid(class_uid: u32) -> Result<MessageDescriptor, PrismError> {
        let pool = OcsfDescriptors::get();

        // STUB: The real implementation queries the pool by the OCSF message name
        // derived from class_uid (e.g., 2004 → "ocsf.DetectionFinding"). Until
        // ocsf-proto-gen is available the pool is empty, so all lookups fail —
        // returning OcsfDescriptorNotFound as required for the Red Gate.
        //
        // The real lookup will be something like:
        //   let msg_name = ocsf_class_uid_to_message_name(class_uid);
        //   pool.get_message_by_name(&msg_name)
        //       .ok_or(PrismError::OcsfDescriptorNotFound { class_uid })
        //
        // We use a placeholder query that always fails with the empty stub pool.
        let _ = pool; // suppress unused warning
        let _ = class_uid;

        // Always fails in the stub — required for Red Gate.
        Err(PrismError::OcsfDescriptorNotFound { class_uid })
    }
}

impl Default for OcsfNormalizer {
    fn default() -> Self {
        Self::new()
    }
}
