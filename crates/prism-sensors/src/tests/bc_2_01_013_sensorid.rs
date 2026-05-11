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
/// GREEN: AdapterRegistry has been re-keyed from (OrgId, SensorType) to (OrgId, SensorId).
/// The registry's `get()` method now accepts `SensorId` directly.
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
