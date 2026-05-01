//! `AdapterRegistry` — maps `(OrgId, SensorType)` to `Arc<dyn SensorAdapter>`.
//!
//! The registry is populated at startup by `init_registry_for_org()` with all
//! four built-in sensor adapters keyed by org identity.  The query engine obtains
//! a shared reference to the registry and calls `get()` to look up the adapter for
//! each fan-out target (BC-2.01.013, AC-3, AC-002).
//!
//! # Multi-Tenant Key
//! The composite `(OrgId, SensorType)` key enforces that adapters for different
//! organisations are structurally segregated — `get(org_a, SensorType::CrowdStrike)`
//! and `get(org_b, SensorType::CrowdStrike)` return independent instances
//! (BC-3.2.001 invariant 1, AC-002).
//!
//! # Thread Safety
//! The registry is read-only after initialization and is `Send + Sync`.
//! It is shared via `Arc<AdapterRegistry>`.
//!
//! Story: S-2.06 | S-3.1.06-ImplPhase | BC: BC-2.01.013, BC-3.2.001

use std::{collections::HashMap, sync::Arc};

use prism_core::{types::SensorType, OrgId};

use crate::adapter::SensorAdapter;

/// Registry mapping `(OrgId, SensorType)` composite keys to `SensorAdapter` instances.
///
/// Populated at process startup with all four built-in sensor adapters per org.
/// After initialization the registry is immutable — adapters are registered
/// once and never removed at runtime.
///
/// The composite key guarantees that a lookup for org A can never return an
/// adapter registered for org B (BC-3.2.001 invariant 1).
#[derive(Default)]
pub struct AdapterRegistry {
    /// Internal store keyed by `(OrgId, SensorType)` composite.
    ///
    /// Stub body: todo!() until `init_registry_for_org` wires org_id through
    /// all adapter constructors (S-3.1.06-ImplPhase AC-002).
    adapters: HashMap<(OrgId, SensorType), Arc<dyn SensorAdapter>>,
}

impl AdapterRegistry {
    /// Creates an empty registry.
    ///
    /// Call `register()` for each adapter (with an explicit `org_id`) before
    /// using the registry in queries.
    pub fn new() -> Self {
        Self {
            adapters: HashMap::new(),
        }
    }

    /// Registers an adapter under the `(org_id, sensor_type)` composite key.
    ///
    /// `sensor_type` is obtained from `adapter.sensor_type()`.
    ///
    /// If an adapter for the same `(org_id, sensor_type)` pair is already
    /// registered, the new adapter replaces the existing one (last-write-wins
    /// within a single org bootstrap sequence, AC-002 EC-002).
    ///
    /// Story: S-3.1.06-ImplPhase | AC-002 | BC-3.2.001 invariant 1
    #[allow(unused_variables)] // stub-phase: org_id + adapter unused until impl (AC-002)
    pub fn register(&mut self, org_id: OrgId, adapter: Arc<dyn SensorAdapter>) {
        todo!(
            "AC-002: store adapter under (org_id, sensor_type) composite key — S-3.1.06-ImplPhase"
        )
    }

    /// Returns a clone of the `Arc<dyn SensorAdapter>` for the
    /// `(org_id, sensor_type)` composite key, or `None` if no adapter is
    /// registered for that pair.
    ///
    /// # AC-001 / EC-001
    /// `get(org_id_A, SensorType::CrowdStrike)` must never return an adapter
    /// registered under `org_id_B` (BC-3.2.001 invariant 1).
    ///
    /// Story: S-3.1.06-ImplPhase | AC-002 | BC-3.2.001 invariant 1
    #[allow(unused_variables)] // stub-phase: org_id + sensor_type unused until impl (AC-002)
    pub fn get(&self, org_id: OrgId, sensor_type: SensorType) -> Option<Arc<dyn SensorAdapter>> {
        todo!("AC-002: look up adapter by (org_id, sensor_type) composite key — S-3.1.06-ImplPhase")
    }

    /// Returns the total number of `(OrgId, SensorType)` entries in the registry.
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
