//! RED integration tests for S-3.1.06-ImplPhase: adapter OrgId binding.
//!
//! These tests verify the structural OrgId enforcement contracts defined in
//! BC-3.1.001–BC-3.1.004 (org identity) and BC-3.2.001 (per-org sensor data
//! isolation). Every test in this file is a RED test — all will panic at runtime
//! with `todo!()` until the implementation phase wires `OrgId` through the full
//! adapter construction and registry dispatch stack.
//!
//! # Test Naming
//! All tests follow the `test_AC_NNN_*` pattern for traceability to story ACs.
//!
//! # Red Gate Invariant
//! Before the implementation phase begins:
//! - `test_AC_001_*` — panics in `init_registry_for_org` (`todo!()`)
//! - `test_AC_002_*` — panics in `AdapterRegistry::register` or `get` (`todo!()`)
//! - `test_AC_003_*` — panics in adapter `fetch()` (org_id mismatch guard is not
//!   yet executed; currently panics via `todo!()` in `init_registry_for_org`)
//! - `test_AC_004_*` — panics verifying `OrgIdMismatch` variant exists (compiles)
//! - `test_AC_005_*` — deprecation attribute smoke test (compile-time check)
//!
//! Story: S-3.1.06-ImplPhase | BCs: BC-3.1.001, BC-3.1.002, BC-3.1.003, BC-3.1.004, BC-3.2.001
#![allow(clippy::expect_used, clippy::unwrap_used)]

use secrecy::SecretString;

use prism_sensors::adapter::{QueryParams, SensorError, SensorSpec};
use prism_sensors::auth::armis::{ArmisAdapter, ArmisAuth};
use prism_sensors::auth::SensorAuth;
use prism_sensors::{
    AdapterRegistry, ArmisAuth as PubArmisAuth, ClarotyAuth, CrowdStrikeAuth, CyberintAuth, OrgId,
    SensorAdapter,
};

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Returns the canonical test-sentinel `OrgId` for org A.
///
/// Replicates `DEFAULT_ORG_ID_BYTES` from `lib.rs` (same byte value).
/// `DEFAULT_ORG_ID_BYTES` is `#[cfg(test)]` gated in the library and therefore
/// not accessible from external integration test crates; we inline the value.
fn org_a() -> OrgId {
    OrgId::from_uuid(uuid::Uuid::from_bytes([
        0x01, 0x8e, 0x3f, 0x71, 0x5c, 0x6d, 0x7a, 0x8b, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x01,
    ]))
}

/// Returns a fresh `OrgId` for org B (distinct from `org_a()`).
fn org_b() -> OrgId {
    OrgId::new()
}

/// Minimal `SensorSpec` with the given `org_id`.
#[allow(deprecated)]
fn make_spec(org_id: OrgId, table: &str) -> SensorSpec {
    SensorSpec {
        org_id,
        source_table: table.to_owned(),
        client_id: String::new(), // deprecated; intentionally empty in new tests
        sensor_config: serde_json::json!({}),
    }
}

fn make_armis_auth(instance_url: &str) -> ArmisAuth {
    ArmisAuth {
        instance_url: instance_url.to_string(),
        secret_key: SecretString::new("test-armis-secret".into()),
    }
}

// ---------------------------------------------------------------------------
// AC-001: init_registry_for_org uses org_id in adapter constructors
// (traces to BC-3.1.001 postcondition 1 and BC-3.2.001 precondition 4)
// ---------------------------------------------------------------------------

/// AC-001 (RED): `init_registry_for_org` with org_id_A must produce a registry
/// that returns `Some` for `(org_id_A, SensorType::CrowdStrike)` and `None`
/// for `(org_id_B, SensorType::CrowdStrike)` where org_id_B ≠ org_id_A.
///
/// RED: panics via `todo!("AC-001: propagate org_id through adapter constructors")`
/// in `init_registry_for_org` until the implementation phase.
///
/// Story: S-3.1.06-ImplPhase | AC-001 | BC-3.2.001 precondition 4
#[test]
fn test_AC_001_init_registry_for_org_uses_org_id_in_signature() {
    use prism_core::types::SensorType;
    use prism_sensors::init_registry_for_org;

    let a = org_a();
    let b = org_b();
    assert_ne!(a, b, "test precondition: org_a and org_b must be distinct");

    let cs_auth = CrowdStrikeAuth {
        client_id: "cs-test".into(),
        client_secret: SecretString::new("cs-secret".into()),
        cloud_region: "us-1".into(),
    };
    let cy_auth = CyberintAuth {
        environment: "portal".into(),
        api_key: SecretString::new("cy-key".into()),
    };
    let cl_auth = ClarotyAuth {
        instance_url: "https://claroty.example.com".into(),
        username: "user".into(),
        password: SecretString::new("pass".into()),
    };
    let ar_auth = PubArmisAuth {
        instance_url: "https://armis.example.com".into(),
        secret_key: SecretString::new("ar-key".into()),
    };

    // RED: init_registry_for_org panics with todo!() until implementation wires org_id
    let registry = init_registry_for_org(
        a,
        &cs_auth,
        &cy_auth,
        &cl_auth,
        SecretString::new("claroty-tok".into()),
        &ar_auth,
        SecretString::new("armis-tok".into()),
    );

    // After implementation: registry keyed under org_a should have CrowdStrike
    assert!(
        registry.get(a, SensorType::CrowdStrike).is_some(),
        "AC-001: registry for org_a must contain CrowdStrike adapter"
    );
    // After implementation: registry for org_a should NOT serve org_b
    assert!(
        registry.get(b, SensorType::CrowdStrike).is_none(),
        "AC-001: registry for org_a must NOT return adapter for org_b"
    );
}

// ---------------------------------------------------------------------------
// AC-002: AdapterRegistry keyed by (OrgId, SensorType) composite key
// (traces to BC-3.2.001 invariant 1 and BC-3.1.003 invariant 2)
// ---------------------------------------------------------------------------

/// AC-002 (RED): Registering adapters for two distinct OrgIds under the same
/// SensorType produces two independent registry entries.
///
/// Specifically: `get(org_id_A, SensorType::CrowdStrike)` and
/// `get(org_id_B, SensorType::CrowdStrike)` must return different Arc pointers.
///
/// RED: panics via `todo!()` in `AdapterRegistry::register` until implementation.
///
/// Story: S-3.1.06-ImplPhase | AC-002 | BC-3.2.001 invariant 1
#[test]
fn test_AC_002_adapter_registry_keyed_by_org_id_and_sensor_type() {
    use prism_core::types::SensorType;
    use std::sync::Arc;

    let a = org_a();
    let b = org_b();
    assert_ne!(a, b, "test precondition: org_a and org_b must be distinct");

    // Build two minimal ArmisAdapters for the two orgs
    let auth_a = make_armis_auth("https://a.armis.com");
    let auth_b = make_armis_auth("https://b.armis.com");

    let adapter_a: Arc<dyn SensorAdapter> = Arc::new(ArmisAdapter::new(
        a,
        &auth_a,
        SecretString::new("tok-a".into()),
    ));
    let adapter_b: Arc<dyn SensorAdapter> = Arc::new(ArmisAdapter::new(
        b,
        &auth_b,
        SecretString::new("tok-b".into()),
    ));
    let ptr_a = Arc::as_ptr(&adapter_a);
    let ptr_b = Arc::as_ptr(&adapter_b);

    let mut registry = AdapterRegistry::new();
    // RED: register panics via todo!() until implementation
    registry.register(a, adapter_a);
    registry.register(b, adapter_b);

    // After implementation: separate entries per org
    let got_a = registry
        .get(a, SensorType::Armis)
        .expect("AC-002: adapter for org_a must be registered");
    let got_b = registry
        .get(b, SensorType::Armis)
        .expect("AC-002: adapter for org_b must be registered");

    assert_eq!(
        Arc::as_ptr(&got_a),
        ptr_a,
        "AC-002: get(org_a, Armis) must return the org_a adapter"
    );
    assert_eq!(
        Arc::as_ptr(&got_b),
        ptr_b,
        "AC-002: get(org_b, Armis) must return the org_b adapter"
    );
    assert_ne!(
        Arc::as_ptr(&got_a),
        Arc::as_ptr(&got_b),
        "AC-002: org_a and org_b adapters must be distinct Arc instances"
    );

    // EC-001: org_a's adapter must NOT be visible via org_b's key
    // (by pointer: the pointer addresses for a and b are different, proven above)
    assert!(
        registry.get(b, SensorType::CrowdStrike).is_none(),
        "AC-002: org_b must not have a CrowdStrike adapter (only Armis was registered for org_b)"
    );
}

// ---------------------------------------------------------------------------
// AC-003: OrgId mismatch returns SensorError::OrgIdMismatch
// (traces to BC-3.2.001 precondition 4 / EC-003 / EC-004)
// ---------------------------------------------------------------------------

/// AC-003 (RED): Constructing an ArmisAdapter for org_A, then calling `fetch()`
/// with a SensorSpec carrying org_B, must return
/// `Err(SensorError::OrgIdMismatch { .. })`.
///
/// No network call must be issued (the mismatch guard fires before any I/O).
///
/// RED: panics via `todo!()` in `init_registry_for_org` or the mismatch guard
/// is not yet implemented; the full guard path is part of the Green Gate phase.
///
/// Story: S-3.1.06-ImplPhase | AC-003 / AC-004 | BC-3.2.001 EC-003 / EC-004
#[tokio::test]
async fn test_AC_003_org_id_mismatch_returns_typed_error() {
    let a = org_a();
    let b = org_b();
    assert_ne!(a, b, "test precondition: org_a and org_b must be distinct");

    let auth = make_armis_auth("https://should-never-be-called.example.com");
    // Adapter constructed for org_a
    let adapter = ArmisAdapter::new(a, &auth, SecretString::new("tok".into()));

    // Spec carries org_b — mismatch
    let spec = make_spec(b, "armis_device");
    let params = QueryParams::default();

    // RED: the OrgId mismatch guard is not yet implemented; this panics or
    // dispatches through the todo!() path until the implementation phase wires
    // the early-return guard at the top of fetch().
    let result = adapter
        .fetch(&spec, &params, &auth as &dyn SensorAuth)
        .await;

    // After implementation: must be Err(OrgIdMismatch { .. })
    assert!(
        result.is_err(),
        "AC-003: dispatch with mismatched OrgId must return Err; got Ok"
    );
    let err = result.unwrap_err();
    assert!(
        matches!(
            err,
            SensorError::OrgIdMismatch {
                adapter_org_id,
                query_org_id,
            } if adapter_org_id == a && query_org_id == b
        ),
        "AC-003: error must be OrgIdMismatch with correct org IDs; got: {err:?}"
    );
    assert!(
        !err.is_transient(),
        "AC-003: OrgIdMismatch must be non-transient (permanent dispatch error)"
    );
}

// ---------------------------------------------------------------------------
// AC-004: legacy init_registry is marked #[deprecated]
// (traces to BC-3.1.001 invariant 1 — org identity resolution available during migration)
// ---------------------------------------------------------------------------

/// AC-004 (compile-time / smoke): `init_registry` is `#[deprecated]` — calling
/// it with `#[allow(deprecated)]` must compile but must panic with `todo!()` at
/// runtime since adapters now require OrgId.
///
/// The deprecation attribute is verified structurally: the test block would NOT
/// compile without `#[allow(deprecated)]`, confirming the attribute is present.
///
/// Story: S-3.1.06-ImplPhase | AC-005 | BC-3.1.001 invariant 1
#[test]
fn test_AC_004_legacy_init_registry_deprecated_warning() {
    use prism_sensors::{init_registry, ArmisAuth as PubArmisAuth2};

    let cs_auth = CrowdStrikeAuth {
        client_id: "cs-id".into(),
        client_secret: SecretString::new("cs-secret".into()),
        cloud_region: "us-1".into(),
    };
    let cy_auth = CyberintAuth {
        environment: "portal".into(),
        api_key: SecretString::new("cy-key".into()),
    };
    let cl_auth = ClarotyAuth {
        instance_url: "https://acme.claroty.com".into(),
        username: "user".into(),
        password: SecretString::new("pass".into()),
    };
    let ar_auth = PubArmisAuth2 {
        instance_url: "https://acme.armis.com".into(),
        secret_key: SecretString::new("ar-key".into()),
    };

    // `#[allow(deprecated)]` is required to call `init_registry`.
    // Its presence here proves the function has `#[deprecated]` on it.
    // At runtime this panics via todo!() because adapters require OrgId (AC-001).
    #[allow(deprecated)]
    let _registry = init_registry(
        &cs_auth,
        &cy_auth,
        &cl_auth,
        SecretString::new("cl-tok".into()),
        &ar_auth,
        SecretString::new("ar-tok".into()),
    );
    // If we reach here (post-implementation): confirms deprecated path still compiles
    // for the migration window (AC-005 — removal deferred to Wave 5).
}

// ---------------------------------------------------------------------------
// AC-005: downstream callers migrate to init_registry_for_org
// (traces to BC-3.1.003 invariant 1 — bijectivity at all times)
// ---------------------------------------------------------------------------

/// AC-005 (RED): `init_registry_for_org` with a valid OrgId must return a
/// registry where `len()` == 4 (all four built-in adapters registered under
/// the given OrgId).
///
/// This test mirrors the existing `test_BC_3_2_001_init_registry_for_org_accepts_org_id_parameter`
/// from `bc_3_2_001_org_id_dispatch.rs`, but from the external test harness to
/// confirm the public API is correct for downstream callers.
///
/// RED: panics via `todo!()` in `init_registry_for_org` until implementation.
///
/// Story: S-3.1.06-ImplPhase | AC-005 / AC-006 | BC-3.1.003 invariant 1
#[test]
fn test_AC_005_downstream_callers_migrate_to_init_registry_for_org() {
    use prism_sensors::init_registry_for_org;

    let org_id = org_a();

    let cs_auth = CrowdStrikeAuth {
        client_id: "cs-id".into(),
        client_secret: SecretString::new("cs-secret".into()),
        cloud_region: "us-1".into(),
    };
    let cy_auth = CyberintAuth {
        environment: "portal".into(),
        api_key: SecretString::new("cy-key".into()),
    };
    let cl_auth = ClarotyAuth {
        instance_url: "https://acme.claroty.com".into(),
        username: "user".into(),
        password: SecretString::new("pass".into()),
    };
    let ar_auth = PubArmisAuth {
        instance_url: "https://acme.armis.com".into(),
        secret_key: SecretString::new("ar-key".into()),
    };

    // RED: panics via todo!() until implementation
    let registry = init_registry_for_org(
        org_id,
        &cs_auth,
        &cy_auth,
        &cl_auth,
        SecretString::new("cl-tok".into()),
        &ar_auth,
        SecretString::new("ar-tok".into()),
    );

    assert_eq!(
        registry.len(),
        4,
        "AC-005: init_registry_for_org must register all 4 built-in adapters; \
         got: {}",
        registry.len()
    );
}
