//! Custom adapter trait and registry (BC-2.16.004).
//!
//! The Rust escape hatch for sensors requiring behavior that cannot be expressed
//! in TOML spec files: exotic auth flows, binary protocols, stateful pagination.
//!
//! - ~80% of sensors: fully config-driven (no adapter needed)
//! - ~20% of sensors: use CustomAdapter for auth or response transformation
//! - The four initial sensors (CrowdStrike, Cyberint, Claroty, Armis) are pure TOML
//!
//! Custom adapter panics are caught via `catch_unwind` and converted to E-SPEC-008.

use prism_core::{PrismError, TenantId};

use crate::spec_parser::FetchStep;
use crate::pipeline::FetchContext;

/// The Rust escape hatch trait for sensors requiring non-declarative parsing.
///
/// Implementors register at startup via `CustomAdapterRegistry`.
/// - `override_auth`: override credential resolution (return None = use spec auth_type)
/// - `override_fetch`: override the HTTP call for a specific step (return None = use spec pipeline)
/// - `transform_response`: transform raw response before spec's response_path extraction
///
/// All overrides are optional — return None to fall through to spec-driven behavior.
pub trait CustomAdapter: Send + Sync {
    /// The sensor_id this adapter handles. Must match a loaded spec file's sensor_id.
    fn sensor_id(&self) -> &str;

    /// Override credential resolution for the given client.
    ///
    /// Return `Some(auth)` to replace spec-declared auth_type.
    /// Return `None` to use spec-declared auth_type (pass-through).
    fn override_auth(&self, client_id: &TenantId) -> Option<Box<dyn SensorAuth>>;

    /// Override the fetch for a specific step.
    ///
    /// Return `Some(records)` to replace the spec-driven HTTP call.
    /// Return `None` to use the spec-driven HTTP call (pass-through).
    ///
    /// Panics in this method are caught by the registry via `catch_unwind` and
    /// converted to `E-SPEC-008` (BC-2.16.004 Error Conditions).
    fn override_fetch(
        &self,
        table: &str,
        step: &FetchStep,
        context: &FetchContext,
    ) -> Option<Vec<serde_json::Value>>;

    /// Transform the raw response before spec's `response_path` extraction.
    ///
    /// Return `Some(transformed)` to replace the raw response.
    /// Return `None` to use the raw response as-is (pass-through).
    fn transform_response(
        &self,
        table: &str,
        raw: &serde_json::Value,
    ) -> Option<serde_json::Value>;
}

/// Placeholder trait for sensor authentication (full definition in prism-sensors).
pub trait SensorAuth: Send + Sync {}

/// Registry for CustomAdapter implementations.
///
/// Adapters are registered in the startup sequence after config loading but before
/// table registration (BC-2.16.004 Registration).
///
/// All methods are `unimplemented!()` — implemented in S-1.11.
pub struct CustomAdapterRegistry {
    _adapters: Vec<Box<dyn CustomAdapter>>,
}

impl CustomAdapterRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        CustomAdapterRegistry {
            _adapters: Vec::new(),
        }
    }

    /// Register a custom adapter.
    ///
    /// Returns Err if an adapter with the same sensor_id is already registered
    /// (EC-003: adapter name must be unique).
    pub fn register(
        &mut self,
        adapter: Box<dyn CustomAdapter>,
    ) -> Result<(), PrismError> {
        unimplemented!("CustomAdapterRegistry::register — implement in S-1.11 (BC-2.16.004)")
    }

    /// Look up an adapter by sensor_id.
    ///
    /// Returns None if no adapter is registered for that sensor_id — the spec
    /// then uses the fully config-driven pipeline (BC-2.16.004 invariant).
    pub fn get(&self, sensor_id: &str) -> Option<&dyn CustomAdapter> {
        unimplemented!("CustomAdapterRegistry::get — implement in S-1.11 (BC-2.16.004)")
    }

    /// Invoke `override_fetch` on the registered adapter for `sensor_id`,
    /// catching any panics and converting them to E-SPEC-008 (BC-2.16.004).
    pub fn safe_override_fetch(
        &self,
        sensor_id: &str,
        table: &str,
        step: &FetchStep,
        context: &FetchContext,
    ) -> Result<Option<Vec<serde_json::Value>>, PrismError> {
        unimplemented!(
            "CustomAdapterRegistry::safe_override_fetch — implement in S-1.11 (BC-2.16.004)"
        )
    }
}

impl Default for CustomAdapterRegistry {
    fn default() -> Self {
        Self::new()
    }
}
