//! Green tests for BC-2.01.013 — SensorId-keyed AdapterRegistry.
//!
//! These tests verify the `AdapterRegistry` interface where lookup
//! uses `SensorId` (open newtype). Migration from `SensorType` (closed enum)
//! is complete as of S-PLUGIN-PREREQ-A.
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
// sensor_type() returns SensorId (open newtype) — migration to SensorId
// is complete as of S-PLUGIN-PREREQ-A.
// ---------------------------------------------------------------------------

use prism_core::SensorId as SensorIdType;

struct SensorIdStubAdapter {
    sensor_id: SensorIdType,
    name: &'static str,
}

#[async_trait]
impl SensorAdapter for SensorIdStubAdapter {
    fn sensor_type(&self) -> SensorIdType {
        self.sensor_id.clone()
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

fn stub_adapter(sensor_id: SensorIdType, name: &'static str) -> Arc<dyn SensorAdapter> {
    Arc::new(SensorIdStubAdapter { sensor_id, name })
}

// ---------------------------------------------------------------------------
// Tests — AC-4 and AC-10 verification
// ---------------------------------------------------------------------------

/// BC-2.01.013 AC-4 + AC-10: AdapterRegistry insert + lookup with SensorId key.
///
/// Verifies BC-2.01.013 postcondition: AdapterRegistry is keyed by (OrgId, SensorId).
/// The registry's `get()` method accepts `SensorId` directly (open newtype dispatch).
///
/// AC-4: HashMap keyed by (OrgId, SensorId)
/// AC-10: register a mock adapter under SensorId::from("crowdstrike"), look it up,
///        assert Some. Verify cross-sensor isolation: lookup for "cyberint" returns None.
#[test]
fn test_BC_2_01_013_004_adapter_registry_sensorid_insert_lookup() {
    let org_id = OrgId::new();
    let mut registry = AdapterRegistry::new();

    // Register a stub adapter using the new SensorId-keyed API.
    registry.register(
        org_id,
        stub_adapter(SensorId::from("crowdstrike"), "crowdstrike"),
    );

    // Lookup via SensorId — now the native API.
    let result = registry.get(org_id, SensorId::from("crowdstrike"));

    assert!(
        result.is_some(),
        "registry.get must return Some for a registered crowdstrike adapter (AC-4, AC-10)"
    );

    // Cross-sensor isolation: looking up "cyberint" must return None when only
    // "crowdstrike" was registered (BC-3.2.001 invariant 1).
    let no_result = registry.get(org_id, SensorId::from("cyberint"));
    assert!(
        no_result.is_none(),
        "registry.get must return None for sensor not registered (cross-sensor isolation, AC-10)"
    );
}
