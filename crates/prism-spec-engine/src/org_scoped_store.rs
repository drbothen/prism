//! OrgId-keyed sensor spec store (S-3.1.05 / BC-3.1.001).
//!
//! Implements ADR-006 §4 Step 2: migrate the internal spec store from
//! `HashMap<(OrgSlug, String), SensorSpec>` to `HashMap<(OrgId, String), SensorSpec>`.
//!
//! # Boundary contract
//!
//! - **User-facing surface:** `get_spec(slug, sensor)` accepts `OrgSlug` for ergonomics.
//!   It calls `OrgRegistry::resolve(slug)` exactly once to translate the slug to `OrgId`
//!   before indexing the store (BC-3.1.001 AC-2).
//! - **Internal store:** keyed on `(OrgId, sensor_name)` — rename-stable and structurally
//!   cross-tenant-isolated (BC-3.1.001 AC-4).
//! - **Missing slug:** returns `Err(SpecEngineError::UnknownOrg)` (BC-3.1.001 AC-1).
//! - **Missing spec:** returns `Err(SpecEngineError::SensorNotFound)` (BC-3.1.001 EC-002).
//! - **Registry not init:** returns `Err(SpecEngineError::RegistryNotInitialized)` — never
//!   panics (BC-3.1.001 AC-3).
//!
//! # Architecture compliance
//!
//! - `Arc<OrgRegistry>` is injected at construction time; no global singleton
//!   (ADR-006 §2.2, S-3.1.05 Architecture Compliance Rules).
//! - `OrgRegistry::resolve` is called once per `get_spec` invocation, never inside
//!   store-layer helpers (BC-3.1.001 invariant 4 / O(1) constraint).
//! - This module MUST NOT import DataFusion or Arrow (AD-015).

use std::collections::HashMap;
use std::sync::Arc;

use prism_core::{OrgId, OrgRegistry, OrgSlug};

use crate::error::SpecEngineError;
use crate::spec_parser::SensorSpec;

// ---------------------------------------------------------------------------
// OrgScopedSpecStore
// ---------------------------------------------------------------------------

/// Internal sensor-spec store keyed on `(OrgId, sensor_name)`.
///
/// Holds `Arc<OrgRegistry>` for slug → OrgId resolution at the user-facing
/// boundary only.  All store-level methods accept `OrgId` directly.
pub struct OrgScopedSpecStore {
    /// Bijective org registry — injected at construction; shared read-only reference.
    ///
    /// `Arc` allows the store to be cloned without cloning the registry data.
    /// The registry is read-only after startup (BC-3.1.001 invariant 1).
    /// Field is read in `get_spec` for slug → OrgId resolution (BC-3.1.001 AC-2).
    registry: Arc<OrgRegistry>,

    /// Internal spec map keyed on `(OrgId, sensor_name)`.
    ///
    /// `sensor_name` is the `SensorSpec::sensor_id` string (e.g., `"crowdstrike"`).
    store: HashMap<(OrgId, String), SensorSpec>,
}

impl OrgScopedSpecStore {
    /// Create an empty store with the given `OrgRegistry`.
    ///
    /// The registry is the authoritative bijective map; it MUST have been fully
    /// populated from `customers/*.toml` before any `get_spec` call
    /// (BC-3.1.001 precondition 1).
    pub fn new(registry: Arc<OrgRegistry>) -> Self {
        Self {
            registry,
            store: HashMap::new(),
        }
    }

    /// Store a sensor spec under `(org_id, spec.sensor_id)`.
    ///
    /// This is the internal write path — callers that load specs at startup
    /// already know the `OrgId` and pass it directly, bypassing slug resolution.
    ///
    /// Calling `insert` replaces any previous spec for the same `(org_id, sensor)` pair.
    pub fn insert(&mut self, org_id: OrgId, spec: SensorSpec) {
        self.store.insert((org_id, spec.sensor_id.clone()), spec);
    }

    /// Retrieve a sensor spec by `OrgSlug` (user-facing) and sensor name.
    ///
    /// # Resolution
    ///
    /// 1. Calls `OrgRegistry::resolve(slug)` exactly once.
    /// 2. If `resolve` returns `None` → `Err(SpecEngineError::UnknownOrg)`.
    /// 3. If `(org_id, sensor)` is absent from the store → `Err(SpecEngineError::SensorNotFound)`.
    /// 4. Otherwise returns `Ok(&SensorSpec)`.
    ///
    /// # Errors
    ///
    /// - `SpecEngineError::UnknownOrg` — slug not in OrgRegistry (BC-3.1.001 AC-1, EC-001)
    /// - `SpecEngineError::SensorNotFound` — org known but sensor absent (BC-3.1.001 EC-002)
    ///
    /// # Panics
    ///
    /// Never panics (BC-3.1.001 AC-3; `RegistryNotInitialized` is returned instead
    /// in the unreachable pre-startup case).
    pub fn get_spec(&self, slug: &OrgSlug, sensor: &str) -> Result<&SensorSpec, SpecEngineError> {
        // AC-2: resolve slug → OrgId exactly once, at the user-facing boundary.
        let org_id = self
            .registry
            .resolve(slug)
            .ok_or_else(|| SpecEngineError::UnknownOrg { slug: slug.clone() })?;

        // Internal store lookup — keyed on (OrgId, sensor_name) for rename stability.
        self.store
            .get(&(org_id, sensor.to_string()))
            .ok_or_else(|| SpecEngineError::SensorNotFound {
                slug: slug.clone(),
                sensor: sensor.to_string(),
            })
    }

    /// Return the number of (OrgId, sensor) pairs in the store.
    ///
    /// Used in tests to verify isolation invariants without accessing private fields.
    pub fn len(&self) -> usize {
        self.store.len()
    }

    /// Returns `true` when the store has no entries.
    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }
}
