//! Integration tests for PLUGIN-MIGRATION-001-A: adapter OrgId binding post-deletion.
//!
//! After PLUGIN-MIGRATION-001-A, all four built-in sensor adapters (CrowdStrike,
//! Cyberint, Claroty, Armis) have been deleted from `prism-sensors`. The
//! `init_registry_for_org` function now accepts only `org_id: OrgId` and returns
//! an empty `AdapterRegistry`. Spec-catalog dispatch (populating the registry from
//! loaded `SensorSpec` catalog via `PluginRegistry`) is wired in `prism-bin` at
//! boot time (BC-2.16.012 INV-SPEC-PARSER-OPEN-001; GAP-002-A deferred to
//! S-WAVE5-PREP-01 / S-3.02-FOLLOWUP-RUNTIME).
//!
//! # Tests retained
//!
//! - `test_AC_001_*` — init_registry_for_org accepts org_id and returns a registry
//! - `test_AC_002_*` — AdapterRegistry (OrgId, SensorId) composite key — uses test
//!   adapter stubs since built-in adapters are deleted
//! - `test_AC_003_*` — OrgIdMismatch enforcement — use a test adapter stub
//! - `test_AC_004_*` — deprecated init_registry() smoke test (new zero-arg signature)
//! - `test_AC_005_*` — init_registry_for_org with valid OrgId returns empty registry
//! - `test_AC_006_*` — OrgId sentinel construction idempotence
//!
//! Story: S-3.1.06-ImplPhase → PLUGIN-MIGRATION-001-A | BCs: BC-3.2.001

#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::sync::Arc;

use arrow::record_batch::RecordBatch;
use async_trait::async_trait;
use prism_core::SensorId;
use prism_sensors::{
    adapter::{FetchOutput, QueryParams, SensorAdapter, SensorError, SensorSpec},
    auth::SensorAuth,
    AdapterRegistry, OrgId,
};

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Returns the canonical test-sentinel `OrgId` for org A.
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

/// Minimal test-only `SensorAuth` impl (all built-in impls deleted in PLUGIN-MIGRATION-001-A).
struct StubAuth;

impl SensorAuth for StubAuth {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn auth_type_name(&self) -> &'static str {
        "custom_via_plugin"
    }
}

/// Minimal test-only `SensorAdapter` for testing registry keying.
///
/// Used in tests that previously used ArmisAdapter (deleted in PLUGIN-MIGRATION-001-A).
struct StubAdapter {
    org_id: OrgId,
    sensor_id: SensorId,
}

#[async_trait]
impl SensorAdapter for StubAdapter {
    fn sensor_type(&self) -> SensorId {
        self.sensor_id.clone()
    }

    fn sensor_name(&self) -> &'static str {
        "stub"
    }

    async fn fetch(
        &self,
        spec: &SensorSpec,
        _params: &QueryParams,
        _auth: &dyn SensorAuth,
    ) -> Result<FetchOutput, SensorError> {
        if spec.org_id != self.org_id {
            return Err(SensorError::OrgIdMismatch {
                adapter_org_id: self.org_id,
                query_org_id: spec.org_id,
            });
        }
        Ok(FetchOutput::new(vec![], false, false))
    }
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

// ---------------------------------------------------------------------------
// AC-001: init_registry_for_org accepts org_id in its signature
// (traces to BC-3.1.001 postcondition 1 and BC-3.2.001 precondition 4)
// ---------------------------------------------------------------------------

/// AC-001: `init_registry_for_org` with org_id_A must produce a registry
/// (even if empty after PLUGIN-MIGRATION-001-A deletion — spec-catalog dispatch
/// populates it in prism-bin at boot time, not here).
///
/// Story: S-3.1.06-ImplPhase → PLUGIN-MIGRATION-001-A AC-004 | BC-3.2.001 precondition 4
#[test]
fn test_AC_001_init_registry_for_org_uses_org_id_in_signature() {
    use prism_sensors::init_registry_for_org;

    let a = org_a();
    let b = org_b();
    assert_ne!(a, b, "test precondition: org_a and org_b must be distinct");

    // After PLUGIN-MIGRATION-001-A: init_registry_for_org takes only org_id.
    // Returns empty registry — spec-catalog dispatch is in prism-bin (GAP-002-A).
    let registry = init_registry_for_org(a);

    // Empty after deletion — spec-catalog dispatch wired in prism-bin (AC-004, GAP-002-A).
    assert_eq!(
        registry.len(),
        0,
        "AC-001: init_registry_for_org post-deletion returns empty registry \
         (spec-catalog dispatch in prism-bin, BC-2.16.012); got len={}",
        registry.len()
    );

    // Cross-org isolation: registry for org_a never serves org_b (empty in both cases).
    assert!(
        registry.get(b, &SensorId::from("crowdstrike")).is_none(),
        "AC-001: registry for org_a must NOT return adapter for org_b"
    );
}

// ---------------------------------------------------------------------------
// AC-002: AdapterRegistry keyed by (OrgId, SensorId) composite key
// (traces to BC-3.2.001 invariant 1 and BC-3.1.003 invariant 2)
// ---------------------------------------------------------------------------

/// AC-002: Registering adapters for two distinct OrgIds under the same
/// SensorId produces two independent registry entries.
///
/// Uses `StubAdapter` since built-in adapters were deleted in PLUGIN-MIGRATION-001-A.
///
/// Story: S-3.1.06-ImplPhase → PLUGIN-MIGRATION-001-A AC-003 | BC-3.2.001 invariant 1
#[test]
fn test_AC_002_adapter_registry_keyed_by_org_id_and_sensor_type() {
    let a = org_a();
    let b = org_b();
    assert_ne!(a, b, "test precondition: org_a and org_b must be distinct");

    // Build two minimal test adapters for the two orgs.
    // (ArmisAdapter deleted in PLUGIN-MIGRATION-001-A; use StubAdapter.)
    let adapter_a: Arc<dyn SensorAdapter> = Arc::new(StubAdapter {
        org_id: a,
        sensor_id: SensorId::from("armis"),
    });
    let adapter_b: Arc<dyn SensorAdapter> = Arc::new(StubAdapter {
        org_id: b,
        sensor_id: SensorId::from("armis"),
    });
    let ptr_a = Arc::as_ptr(&adapter_a);
    let ptr_b = Arc::as_ptr(&adapter_b);

    let mut registry = AdapterRegistry::new();
    registry.register(a, adapter_a);
    registry.register(b, adapter_b);

    // After registration: separate entries per org.
    let got_a = registry
        .get(a, &SensorId::from("armis"))
        .expect("AC-002: adapter for org_a must be registered");
    let got_b = registry
        .get(b, &SensorId::from("armis"))
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

    // EC-001: org_a's adapter must NOT be visible via org_b's key.
    assert!(
        registry.get(b, &SensorId::from("crowdstrike")).is_none(),
        "AC-002: org_b must not have a CrowdStrike adapter (only Armis stub was registered for org_b)"
    );
}

// ---------------------------------------------------------------------------
// AC-003: OrgId mismatch returns SensorError::OrgIdMismatch
// (traces to BC-3.2.001 precondition 4 / EC-003 / EC-004)
// ---------------------------------------------------------------------------

/// AC-003: Constructing a StubAdapter for org_A, then calling `fetch()`
/// with a SensorSpec carrying org_B, must return
/// `Err(SensorError::OrgIdMismatch { .. })`.
///
/// Uses `StubAdapter` since ArmisAdapter was deleted in PLUGIN-MIGRATION-001-A.
///
/// Story: S-3.1.06-ImplPhase → PLUGIN-MIGRATION-001-A | BC-3.2.001 EC-003 / EC-004
#[tokio::test]
async fn test_AC_003_org_id_mismatch_returns_typed_error() {
    let a = org_a();
    let b = org_b();
    assert_ne!(a, b, "test precondition: org_a and org_b must be distinct");

    // Adapter constructed for org_a.
    let adapter = StubAdapter {
        org_id: a,
        sensor_id: SensorId::from("stub"),
    };

    // Spec carries org_b — mismatch.
    let spec = make_spec(b, "stub_table");
    let params = QueryParams::default();
    let auth = StubAuth;

    let result = adapter
        .fetch(&spec, &params, &auth as &dyn SensorAuth)
        .await;

    assert!(
        result.is_err(),
        "AC-003: dispatch with mismatched OrgId must return Err; got Ok"
    );
    let err = result.unwrap_err();
    assert!(
        matches!(
            err,
            SensorError::OrgIdMismatch {
                ref adapter_org_id,
                ref query_org_id,
            } if *adapter_org_id == a && *query_org_id == b
        ),
        "AC-003: error must be SensorError::OrgIdMismatch {{ adapter_org_id: {a}, query_org_id: {b} }}; \
         got: {err:?}"
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

/// AC-004 (smoke): `init_registry` is `#[deprecated]` — calling it with
/// `#[allow(deprecated)]` must compile and return an empty registry.
///
/// After PLUGIN-MIGRATION-001-A: init_registry() takes NO arguments (all
/// built-in adapter credential parameters removed).
///
/// Story: PLUGIN-MIGRATION-001-A AC-003/AC-005 | BC-3.1.001 invariant 1
#[test]
fn test_AC_004_legacy_init_registry_deprecated_warning() {
    use prism_sensors::init_registry;

    // `#[allow(deprecated)]` is required to call `init_registry`.
    // Its presence here proves the function has `#[deprecated]` on it.
    // After PLUGIN-MIGRATION-001-A: zero arguments, returns empty registry.
    #[allow(deprecated)]
    let registry = init_registry();

    // Empty registry — spec-catalog dispatch is in prism-bin.
    assert_eq!(
        registry.len(),
        0,
        "AC-004: deprecated init_registry post-deletion returns empty registry"
    );
}

// ---------------------------------------------------------------------------
// AC-005: downstream callers migrate to init_registry_for_org
// (traces to BC-3.1.003 invariant 1 — bijectivity at all times)
// ---------------------------------------------------------------------------

/// AC-005: `init_registry_for_org` with a valid OrgId returns an empty registry.
///
/// After PLUGIN-MIGRATION-001-A: all four built-in adapters are deleted;
/// registry is populated by spec-catalog dispatch in prism-bin at boot time
/// (GAP-002-A; S-WAVE5-PREP-01 / S-3.02-FOLLOWUP-RUNTIME).
///
/// Story: PLUGIN-MIGRATION-001-A AC-004 | BC-3.1.003 invariant 1
#[test]
fn test_AC_005_downstream_callers_migrate_to_init_registry_for_org() {
    use prism_sensors::init_registry_for_org;

    let org_id = org_a();

    // After PLUGIN-MIGRATION-001-A: init_registry_for_org takes only org_id.
    let registry = init_registry_for_org(org_id);

    // Empty after deletion (spec-catalog dispatch in prism-bin, GAP-002-A).
    assert_eq!(
        registry.len(),
        0,
        "AC-005: init_registry_for_org post-deletion must return empty registry; got: {}",
        registry.len()
    );
}

// ---------------------------------------------------------------------------
// AC-006: test callers construct OrgId from the DEFAULT_ORG_ID_BYTES constant
// (traces to BC-3.1.003 invariant 1 — bijectivity holds at all times)
// ---------------------------------------------------------------------------

/// AC-006: `init_registry_for_org` called with the canonical sentinel
/// `OrgId` (derived from the same bytes as `DEFAULT_ORG_ID_BYTES`) must produce
/// a registry (empty post-deletion).
///
/// OrgId construction from sentinel bytes remains idempotent.
///
/// Story: PLUGIN-MIGRATION-001-A AC-004 | BC-3.1.003 invariant 1
#[test]
#[allow(clippy::similar_names)]
fn test_AC_006_test_callers_use_OrgId_from_const_helper() {
    let sentinel_bytes: [u8; 16] = [
        0x01, 0x8e, 0x3f, 0x71, 0x5c, 0x6d, 0x7a, 0x8b, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x01,
    ];

    // OrgId construction from sentinel bytes is idempotent.
    let sentinel_1 = OrgId::from_uuid(uuid::Uuid::from_bytes(sentinel_bytes));
    let sentinel_2 = OrgId::from_uuid(uuid::Uuid::from_bytes(sentinel_bytes));
    assert_eq!(
        sentinel_1, sentinel_2,
        "AC-006: OrgId construction from sentinel bytes must be idempotent"
    );
    assert_eq!(
        sentinel_1,
        org_a(),
        "AC-006: sentinel OrgId from inlined bytes must equal org_a()"
    );

    use prism_sensors::init_registry_for_org;

    // After PLUGIN-MIGRATION-001-A: init_registry_for_org takes only org_id.
    let registry = init_registry_for_org(sentinel_1);

    // Empty after deletion (spec-catalog dispatch in prism-bin, GAP-002-A).
    assert_eq!(
        registry.len(),
        0,
        "AC-006: init_registry_for_org with sentinel OrgId post-deletion returns empty registry; got: {}",
        registry.len()
    );
}
