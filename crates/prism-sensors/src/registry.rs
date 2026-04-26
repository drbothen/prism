//! `AdapterRegistry` — maps `SensorType` to `Arc<dyn SensorAdapter>`.
//!
//! The registry is populated at startup with all four built-in sensor adapters
//! (CrowdStrike, Cyberint, Claroty, Armis). The query engine obtains a shared
//! reference to the registry and calls `get()` to look up the adapter for each
//! fan-out target (BC-2.01.013, AC-3).
//!
//! # Thread Safety
//! The registry is read-only after initialization and is `Send + Sync`.
//! It is shared via `Arc<AdapterRegistry>`.
//!
//! Story: S-2.06 | BC: BC-2.01.013

use std::{collections::HashMap, sync::Arc};

use prism_core::types::SensorType;

use crate::adapter::SensorAdapter;

/// Registry mapping `SensorType` keys to their `SensorAdapter` implementations.
///
/// Populated at process startup with all four built-in sensor adapters.
/// After initialization the registry is immutable — adapters are registered
/// once and never removed at runtime.
#[derive(Default)]
pub struct AdapterRegistry {
    adapters: HashMap<SensorType, Arc<dyn SensorAdapter>>,
}

impl AdapterRegistry {
    /// Creates an empty registry.
    ///
    /// Call `register()` for each adapter before using the registry in queries.
    pub fn new() -> Self {
        Self {
            adapters: HashMap::new(),
        }
    }

    /// Registers an adapter by its declared `SensorType`.
    ///
    /// If an adapter for the same `SensorType` is already registered, the new
    /// adapter replaces the existing one.
    ///
    /// # AC-3
    /// After `register(adapter)` for `SensorType::CrowdStrike`, calling
    /// `get(SensorType::CrowdStrike)` returns the same instance (by `Arc` pointer).
    pub fn register(&mut self, adapter: Arc<dyn SensorAdapter>) {
        let sensor_type = adapter.sensor_type();
        self.adapters.insert(sensor_type, adapter);
    }

    /// Returns a clone of the `Arc<dyn SensorAdapter>` for `sensor_type`, or
    /// `None` if no adapter is registered.
    ///
    /// The returned `Arc` shares ownership with the registry.
    pub fn get(&self, sensor_type: SensorType) -> Option<Arc<dyn SensorAdapter>> {
        self.adapters.get(&sensor_type).cloned()
    }

    /// Returns the number of adapters currently registered.
    pub fn len(&self) -> usize {
        self.adapters.len()
    }

    /// Returns `true` if no adapters are registered.
    pub fn is_empty(&self) -> bool {
        self.adapters.is_empty()
    }
}

impl std::fmt::Debug for AdapterRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let keys: Vec<_> = self.adapters.keys().collect();
        f.debug_struct("AdapterRegistry")
            .field("registered_sensors", &keys)
            .finish()
    }
}
