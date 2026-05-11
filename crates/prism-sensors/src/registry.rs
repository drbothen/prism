//! `AdapterRegistry` — maps `(OrgId, SensorId)` to `Arc<dyn SensorAdapter>`.
//!
//! The registry is populated at startup by `init_registry_for_org()` with all
//! built-in sensor adapters keyed by org identity.  The query engine obtains
//! a shared reference to the registry and calls `get()` to look up the adapter for
//! each fan-out target (BC-2.01.013, AC-3, AC-002).
//!
//! # Multi-Tenant Key
//! The composite `(OrgId, SensorId)` key enforces that adapters for different
//! organisations are structurally segregated — `get(org_a, SensorId::from("crowdstrike"))`
//! and `get(org_b, SensorId::from("crowdstrike"))` return independent instances
//! (BC-3.2.001 invariant 1, AC-002).
//!
//! # Thread Safety
//! The registry is read-only after initialization and is `Send + Sync`.
//! It is shared via `Arc<AdapterRegistry>`.
//!
//! Story: S-2.06 | S-3.1.06-ImplPhase | S-PLUGIN-PREREQ-A | BC: BC-2.01.013, BC-3.2.001

use std::{collections::HashMap, sync::Arc};

use prism_core::{OrgId, SensorId};

use crate::adapter::SensorAdapter;

/// Registry mapping `(OrgId, SensorId)` composite keys to `SensorAdapter` instances.
///
/// Populated at process startup with all built-in sensor adapters per org.
/// After initialization the registry is immutable — adapters are registered
/// once and never removed at runtime.
///
/// The composite key guarantees that a lookup for org A can never return an
/// adapter registered for org B (BC-3.2.001 invariant 1).
#[derive(Default)]
pub struct AdapterRegistry {
    /// Internal store keyed by `(OrgId, SensorId)` composite.
    ///
    /// Populated via `register()`. The composite key guarantees that a lookup
    /// for org A cannot return an adapter registered for org B.
    adapters: HashMap<(OrgId, SensorId), Arc<dyn SensorAdapter>>,
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

    /// Registers an adapter under the `(org_id, sensor_id)` composite key.
    ///
    /// `sensor_id` is obtained from `adapter.sensor_type()`.
    ///
    /// If an adapter for the same `(org_id, sensor_id)` pair is already
    /// registered, the new adapter replaces the existing one (last-write-wins
    /// within a single org bootstrap sequence, AC-002 EC-002).
    ///
    /// Story: S-3.1.06-ImplPhase | S-PLUGIN-PREREQ-A | AC-002 | BC-3.2.001 invariant 1
    pub fn register(&mut self, org_id: OrgId, adapter: Arc<dyn SensorAdapter>) {
        let sensor_id = adapter.sensor_type();
        self.adapters.insert((org_id, sensor_id), adapter);
    }

    /// Returns a clone of the `Arc<dyn SensorAdapter>` for the
    /// `(org_id, sensor_id)` composite key, or `None` if no adapter is
    /// registered for that pair.
    ///
    /// # AC-001 / EC-001
    /// `get(org_id_A, SensorId::from("crowdstrike"))` must never return an adapter
    /// registered under `org_id_B` (BC-3.2.001 invariant 1).
    ///
    /// Story: S-3.1.06-ImplPhase | S-PLUGIN-PREREQ-A | AC-002 | BC-3.2.001 invariant 1
    pub fn get(&self, org_id: OrgId, sensor_id: SensorId) -> Option<Arc<dyn SensorAdapter>> {
        self.adapters.get(&(org_id, sensor_id)).cloned()
    }

    /// Returns all adapters registered for the given sensor id, regardless of org.
    ///
    /// Used by the materialization pipeline when an OrgSlug→OrgId mapping is not
    /// available (MVP: single-org adapters registered without strict org binding).
    /// Returns adapter + org_id pairs so callers can attribute results correctly.
    ///
    /// # Multi-tenant note
    /// Production use MUST migrate to `get(org_id, sensor_id)` once the
    /// OrgRegistry (OrgSlug→OrgId mapping) is wired (S-WAVE5-PREP-01 §Boot step 3).
    pub fn get_all_for_sensor_type(
        &self,
        sensor_id: SensorId,
    ) -> Vec<(OrgId, Arc<dyn SensorAdapter>)> {
        self.adapters
            .iter()
            .filter(|((_, sid), _)| *sid == sensor_id)
            .map(|((org_id, _), adapter)| (*org_id, Arc::clone(adapter)))
            .collect()
    }

    /// Returns all adapters registered for the given sensor id, regardless of org.
    ///
    /// Alias for `get_all_for_sensor_type` using the new naming convention.
    /// Provided for forward-compatibility with callers migrated to `SensorId`.
    pub fn get_all_for_sensor(&self, sensor_id: &SensorId) -> Vec<(OrgId, Arc<dyn SensorAdapter>)> {
        self.adapters
            .iter()
            .filter(|((_, sid), _)| sid == sensor_id)
            .map(|((org_id, _), adapter)| (*org_id, Arc::clone(adapter)))
            .collect()
    }

    /// Look up an adapter by `(org_id, sensor_id)` where `sensor_id` is a string key.
    ///
    /// Convenience accessor for callers that hold a `SensorId` reference.
    /// Equivalent to `get(org_id, sensor_id.clone())` but avoids cloning for lookup.
    pub fn get_by_id(&self, org_id: OrgId, sensor_id: SensorId) -> Option<Arc<dyn SensorAdapter>> {
        self.adapters.get(&(org_id, sensor_id)).cloned()
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
