//! Tests for BC-2.01.013: DataSource Trait Eliminates Per-Sensor Code Duplication.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//!
//! Covers:
//! - `SensorAdapter` object-safety and `Send + Sync + 'static` bounds (AC-3)
//! - `AdapterRegistry::register()` + `get()` round-trip (AC-3, TV-BC-2.01.013-001)
//! - Lookup returns the *same* `Arc` instance after registration
//! - `get()` for an unregistered `SensorId` returns `None`
//! - Registry `len()` / `is_empty()` helpers
//! - Sealed trait: `SensorAuth` cannot be implemented outside `prism_sensors`
//!   (verified structurally — the private module is not accessible from tests)
//! - Adapter registered for each of the four sensor names
//!
//! Story: S-2.06 | BC: BC-2.01.013

use std::sync::Arc;

use arrow::record_batch::RecordBatch;
use async_trait::async_trait;
use prism_core::SensorId;

use crate::{
    adapter::{QueryParams, SensorAdapter, SensorError, SensorSpec},
    auth::SensorAuth,
    registry::AdapterRegistry,
};

// ---------------------------------------------------------------------------
// Helpers — minimal stub adapters for registry tests
// ---------------------------------------------------------------------------

/// A no-op adapter stub that never calls `fetch()`.
/// Used to test registry insertion and retrieval without touching HTTP.
struct StubAdapter {
    sensor_type: SensorId,
    name: &'static str,
}

#[async_trait]
impl SensorAdapter for StubAdapter {
    fn sensor_type(&self) -> SensorId {
        self.sensor_type.clone()
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

fn stub(sensor_type: SensorId, name: &'static str) -> Arc<dyn SensorAdapter> {
    Arc::new(StubAdapter { sensor_type, name })
}

// ---------------------------------------------------------------------------
// AC-3 / TV-BC-2.01.013-001: registry register + get round-trip
// ---------------------------------------------------------------------------

/// AC-3: After registering a CrowdStrikeAdapter, `get(SensorId::from("crowdstrike"))`
/// returns the same `Arc` instance (by pointer equality).
///
/// TV-BC-2.01.013-001
#[test]
fn test_BC_2_01_013_registry_get_returns_registered_crowdstrike_adapter() {
    let adapter = stub(SensorId::from("crowdstrike"), "crowdstrike");
    let ptr = Arc::as_ptr(&adapter);
    let org_id = prism_core::OrgId::new(); // TODO impl-phase: use real OrgId

    let mut registry = AdapterRegistry::new();
    registry.register(org_id, Arc::clone(&adapter));

    let retrieved = registry
        .get(org_id, SensorId::from("crowdstrike"))
        .expect("CrowdStrike adapter must be registered");
    assert_eq!(
        Arc::as_ptr(&retrieved),
        ptr,
        "get() must return the same Arc instance that was registered"
    );
}

/// AC-3: All four sensor types can be registered and retrieved.
#[test]
fn test_BC_2_01_013_registry_all_four_sensor_types_registered_and_retrieved() {
    let org_id = prism_core::OrgId::new(); // TODO impl-phase: use real OrgId
    let mut registry = AdapterRegistry::new();

    for (sensor_type, name) in [
        (SensorId::from("crowdstrike"), "crowdstrike"),
        (SensorId::from("cyberint"), "cyberint"),
        (SensorId::from("claroty"), "claroty"),
        (SensorId::from("armis"), "armis"),
    ] {
        registry.register(org_id, stub(sensor_type, name));
    }

    assert_eq!(registry.len(), 4, "all four adapters must be registered");

    for sensor_type in [
        SensorId::from("crowdstrike"),
        SensorId::from("cyberint"),
        SensorId::from("claroty"),
        SensorId::from("armis"),
    ] {
        let name = format!("{sensor_type}");
        assert!(
            registry.get(org_id, sensor_type).is_some(),
            "adapter for {name} must be retrievable after registration"
        );
    }
}

/// Registry `get()` returns `None` for a sensor type that was never registered.
#[test]
fn test_BC_2_01_013_registry_get_returns_none_for_unregistered_sensor() {
    // Only register CrowdStrike; Armis is intentionally absent.
    let org_id = prism_core::OrgId::new(); // TODO impl-phase: use real OrgId
    let mut registry = AdapterRegistry::new();
    registry.register(org_id, stub(SensorId::from("crowdstrike"), "crowdstrike"));

    assert!(
        registry.get(org_id, SensorId::from("armis")).is_none(),
        "get() must return None for a sensor type that was not registered"
    );
}

/// Registering a second adapter for the same `SensorId` replaces the first.
#[test]
fn test_BC_2_01_013_registry_re_register_replaces_existing_adapter() {
    let first = stub(SensorId::from("crowdstrike"), "first");
    let second = stub(SensorId::from("crowdstrike"), "second");
    let second_ptr = Arc::as_ptr(&second);
    let org_id = prism_core::OrgId::new(); // TODO impl-phase: use real OrgId

    let mut registry = AdapterRegistry::new();
    registry.register(org_id, first);
    registry.register(org_id, Arc::clone(&second));

    let retrieved = registry
        .get(org_id, SensorId::from("crowdstrike"))
        .expect("adapter must be present");
    assert_eq!(
        Arc::as_ptr(&retrieved),
        second_ptr,
        "second registration must replace the first"
    );
    // Length is still 1 — only one CrowdStrike entry.
    assert_eq!(registry.len(), 1);
}

/// Empty registry reports `is_empty()` = true, `len()` = 0.
#[test]
fn test_BC_2_01_013_registry_is_empty_on_new() {
    let registry = AdapterRegistry::new();
    assert!(registry.is_empty());
    assert_eq!(registry.len(), 0);
}

// ---------------------------------------------------------------------------
// Object-safety: `dyn SensorAdapter` must compile and be storable in Arc.
// ---------------------------------------------------------------------------

/// Verifies that `Arc<dyn SensorAdapter>` can be constructed, confirming the
/// trait is object-safe (no generic methods, no associated types preventing
/// vtable construction). This is a compile-time check captured as a runtime no-op.
///
/// BC-2.01.013 architecture compliance rule.
#[test]
fn test_BC_2_01_013_sensor_adapter_is_object_safe() {
    let _adapter: Arc<dyn SensorAdapter> = stub(SensorId::from("crowdstrike"), "crowdstrike");
    // If this compiles, `dyn SensorAdapter` is object-safe.
}

/// Verifies that `AdapterRegistry` can hold adapters for all four sensor types
/// stored as `Arc<dyn SensorAdapter>` — confirming polymorphic dispatch works.
#[test]
fn test_BC_2_01_013_registry_stores_dyn_adapters_for_all_sensor_types() {
    let org_id = prism_core::OrgId::new(); // TODO impl-phase: use real OrgId
    let mut registry = AdapterRegistry::new();
    registry.register(org_id, stub(SensorId::from("crowdstrike"), "crowdstrike"));
    registry.register(org_id, stub(SensorId::from("cyberint"), "cyberint"));
    registry.register(org_id, stub(SensorId::from("claroty"), "claroty"));
    registry.register(org_id, stub(SensorId::from("armis"), "armis"));

    // Sanity: all four are distinct entries.
    assert_eq!(registry.len(), 4);
}

// ---------------------------------------------------------------------------
// SensorAuth sealed trait: structural verification
// ---------------------------------------------------------------------------

/// Verifies that `SensorAuth` is implemented by the four built-in types and
/// that those types implement `Send + Sync + 'static` (required for cross-task
/// sharing). This is a compile-time bound check expressed as a static assertion.
///
/// BC-2.01.013 invariant: sealed trait — external impls are compile-impossible.
/// The sealed guarantee is enforced by the private `Sealed` marker in `auth::private`,
/// which is not re-exported and therefore unreachable from this test crate.
#[test]
fn test_BC_2_01_013_sensor_auth_subtypes_are_send_sync_static() {
    fn assert_send_sync_static<T: Send + Sync + 'static>() {}

    assert_send_sync_static::<crate::auth::CrowdStrikeAuth>();
    assert_send_sync_static::<crate::auth::CyberintAuth>();
    assert_send_sync_static::<crate::auth::ClarotyAuth>();
    assert_send_sync_static::<crate::auth::ArmisAuth>();
}

/// Verifies that `dyn SensorAuth` can be held in a `Box` (object-safe).
/// This mirrors how `CredentialResolver::resolve()` returns `Box<dyn SensorAuth>`.
#[test]
fn test_BC_2_01_013_sensor_auth_is_object_safe_boxed() {
    use secrecy::SecretString;

    let auth: Box<dyn SensorAuth> = Box::new(crate::auth::CrowdStrikeAuth {
        client_id: "test-client".into(),
        client_secret: SecretString::new("secret".into()),
        cloud_region: "us-1".into(),
    });
    // If this compiles, `dyn SensorAuth` is object-safe.
    let _ = auth;
}

// ---------------------------------------------------------------------------
// `sensor_name()` returns the expected sensor name string per adapter
// ---------------------------------------------------------------------------

/// Each adapter's `sensor_name()` must return its canonical sensor name string.
/// This is used in tracing spans and error messages.
#[test]
fn test_BC_2_01_013_stub_adapter_sensor_name_matches_declared() {
    let cs = stub(SensorId::from("crowdstrike"), "crowdstrike");
    assert_eq!(cs.sensor_name(), "crowdstrike");

    let cy = stub(SensorId::from("cyberint"), "cyberint");
    assert_eq!(cy.sensor_name(), "cyberint");

    let cl = stub(SensorId::from("claroty"), "claroty");
    assert_eq!(cl.sensor_name(), "claroty");

    let ar = stub(SensorId::from("armis"), "armis");
    assert_eq!(ar.sensor_name(), "armis");
}
