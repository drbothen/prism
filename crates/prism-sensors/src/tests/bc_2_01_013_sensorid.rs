//! Red Gate tests for BC-2.01.013 — SensorId-keyed AdapterRegistry.
//!
//! These tests exercise the NEW `AdapterRegistry` interface where lookup
//! uses `SensorId` (open newtype) instead of `SensorType` (closed enum).
//!
//! # Red Gate status
//! ALL tests in this module MUST FAIL before implementation. They fail because
//! `AdapterRegistry` currently keys by `SensorType`, not `SensorId`. The
//! `get_by_sensor_id()` bridge function in this module panics at `todo!()`
//! to represent the unimplemented migration.
//!
//! # Story: S-PLUGIN-PREREQ-A
//! # BC: BC-2.01.013 — DataSource Trait: Spec-Driven Adapter Pattern (AC-4, AC-10)

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use arrow::record_batch::RecordBatch;
use async_trait::async_trait;
use prism_core::{OrgId, SensorId};

use crate::{
    adapter::{QueryParams, SensorAdapter, SensorError, SensorSpec},
    auth::SensorAuth,
    registry::AdapterRegistry,
};

// ---------------------------------------------------------------------------
// Stub adapter — minimal SensorAdapter impl for registry wiring tests.
// Keeps existing trait method signatures so this file compiles today
// (sensor_type() still returns SensorType per the current trait definition).
// ---------------------------------------------------------------------------

use prism_core::types::SensorType;

struct SensorIdStubAdapter {
    sensor_type: SensorType,
    name: &'static str,
}

#[async_trait]
impl SensorAdapter for SensorIdStubAdapter {
    fn sensor_type(&self) -> SensorType {
        self.sensor_type
    }

    fn sensor_name(&self) -> &'static str {
        self.name
    }

    async fn fetch(
        &self,
        _spec: &SensorSpec,
        _params: &QueryParams,
        _auth: &dyn SensorAuth,
    ) -> Result<Vec<RecordBatch>, SensorError> {
        Err(SensorError::Internal {
            detail: "stub — not implemented".into(),
        })
    }
}

fn stub_adapter(sensor_type: SensorType, name: &'static str) -> Arc<dyn SensorAdapter> {
    Arc::new(SensorIdStubAdapter { sensor_type, name })
}

// ---------------------------------------------------------------------------
// Bridge function — represents the future SensorId-keyed lookup API.
//
// This function DOES NOT EXIST on AdapterRegistry yet.
// It is placed here as a `todo!()` stub so the test compiles but fails at
// the correct site — proving the Red Gate wires up to the unimplemented path.
//
// Implementer will: (a) change AdapterRegistry's HashMap key from (OrgId, SensorType)
// to (OrgId, SensorId), and (b) update register() + get() signatures accordingly.
// This stub will then be replaced with a direct AdapterRegistry::get() call.
// ---------------------------------------------------------------------------

/// Bridge stub: lookup adapter by (OrgId, SensorId).
///
/// Panics at todo!() — the AdapterRegistry does not yet support SensorId keys.
/// This represents the post-migration API that the implementer must build.
fn get_by_sensor_id(
    _registry: &AdapterRegistry,
    _org_id: OrgId,
    _sensor_id: SensorId,
) -> Option<Arc<dyn SensorAdapter>> {
    todo!(
        "S-PLUGIN-PREREQ-A: AdapterRegistry must be re-keyed from (OrgId, SensorType) \
        to (OrgId, SensorId). Implement by changing registry.rs HashMap key type and \
        updating get() signature. See AC-4 and AC-10."
    )
}

// ---------------------------------------------------------------------------
// Red Gate tests
// ---------------------------------------------------------------------------

/// BC-2.01.013 AC-4 + AC-10: AdapterRegistry insert + lookup with SensorId key.
///
/// Red Gate: panics at todo!() in get_by_sensor_id() — the bridge stub for the
/// not-yet-implemented SensorId-keyed lookup path.
///
/// Post-implementation: this test will call AdapterRegistry::get(org_id, SensorId::from("crowdstrike"))
/// directly once the registry key type is migrated.
#[test]
fn test_BC_2_01_013_004_adapter_registry_sensorid_insert_lookup() {
    let org_id = OrgId::new();
    let mut registry = AdapterRegistry::new();

    // Register a stub CrowdStrike adapter using the current (SensorType-keyed) API.
    // Post-migration: this will use SensorId directly.
    registry.register(org_id, stub_adapter(SensorType::CrowdStrike, "crowdstrike"));

    // The lookup via SensorId — this is the NEW API that doesn't exist yet.
    // Panics at todo!() proving the Red Gate.
    let result = get_by_sensor_id(&registry, org_id, SensorId::from("crowdstrike"));

    assert!(
        result.is_some(),
        "get_by_sensor_id must return Some for a registered crowdstrike adapter"
    );

    // Cross-sensor isolation: looking up "cyberint" must return None when only
    // "crowdstrike" was registered.
    let no_result = get_by_sensor_id(&registry, org_id, SensorId::from("cyberint"));
    assert!(
        no_result.is_none(),
        "get_by_sensor_id must return None for sensor not registered (cross-sensor isolation)"
    );
}
