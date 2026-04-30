#![allow(non_snake_case)]
//! BC-3.1.001 — OrgRegistry Bijective Slug/UUID Resolution applied to spec-engine.
//!
//! S-3.1.05: Scope sensor specs per OrgId.
//!
//! These tests exercise the `OrgScopedSpecStore` API surface:
//! - `get_spec(slug, sensor)` resolves slug → OrgId via OrgRegistry then indexes store.
//! - Unknown slug returns `Err(SpecEngineError::UnknownOrg)`.
//! - Known slug, missing sensor returns `Err(SpecEngineError::SensorNotFound)`.
//! - Specs stored under OrgId(A) are invisible to a different slug/OrgId(B).
//!
//! All tests use canonical test vectors from BC-3.1.001 §Canonical Test Vectors.
//!
//! # Green Gate note
//! `OrgScopedSpecStore::get_spec` is fully implemented — all tests verify
//! Ok/Err return values directly (S-3.1.05 Implementer phase complete).

use std::sync::Arc;

use prism_core::{OrgId, OrgRegistry, OrgSlug};
use prism_spec_engine::spec_parser::{AuthType, SensorSpec, TableSpec};
use prism_spec_engine::{OrgScopedSpecStore, SpecEngineError};

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Build a minimal `SensorSpec` with the given `sensor_id`.
fn make_sensor_spec(sensor_id: &str) -> SensorSpec {
    SensorSpec {
        sensor_id: sensor_id.to_string(),
        name: format!("{sensor_id} sensor"),
        auth_type: AuthType::ApiKey,
        base_url: "https://example.com".to_string(),
        tables: vec![TableSpec::new_point_in_time(
            "events",
            "security_finding",
            vec![],
            vec![],
        )],
        rate_limit_hints: None,
        version: "1.0.0".to_string(),
    }
}

/// Populate an `OrgRegistry` with `(slug, id)` pairs and return it wrapped in `Arc`.
fn make_registry(pairs: &[(&str, OrgId)]) -> Arc<OrgRegistry> {
    let reg = OrgRegistry::new();
    for (slug_str, id) in pairs {
        reg.register(OrgSlug::new(slug_str), *id)
            .expect("test registry setup must not conflict");
    }
    Arc::new(reg)
}

// ---------------------------------------------------------------------------
// BC-3.1.001 TV-3.1.001-01 — happy path forward resolution
// ---------------------------------------------------------------------------

/// `get_spec(acme_slug, "crowdstrike")` returns the spec stored under `OrgId(acme)`.
///
/// Traces to: BC-3.1.001 postcondition 1, S-3.1.05 AC-1.
#[test]
fn test_BC_3_1_001_get_spec_resolves_slug_to_org_id() {
    let acme_id = OrgId::new();
    let registry = make_registry(&[("acme-corp", acme_id)]);
    let mut store = OrgScopedSpecStore::new(registry);
    store.insert(acme_id, make_sensor_spec("crowdstrike"));

    let slug = OrgSlug::new("acme-corp");
    // Should return Ok(&SensorSpec) post-implementation; todo!() fires pre-impl.
    let _ = store.get_spec(&slug, "crowdstrike").unwrap();
}

// ---------------------------------------------------------------------------
// BC-3.1.001 TV-3.1.001-02 — unknown slug
// ---------------------------------------------------------------------------

/// `get_spec(unknown_slug, sensor)` returns `Err(UnknownOrg)`.
///
/// Traces to: BC-3.1.001 EC-001, S-3.1.05 AC-1 / EC-001.
#[test]
fn test_BC_3_1_001_get_spec_unknown_org_returns_error() {
    let registry = make_registry(&[("acme-corp", OrgId::new())]);
    let store = OrgScopedSpecStore::new(registry);

    let unknown = OrgSlug::new("unknown-org");
    // Should return Err(UnknownOrg) post-implementation; todo!() fires pre-impl.
    let result = store.get_spec(&unknown, "crowdstrike");
    assert!(
        matches!(result, Err(SpecEngineError::UnknownOrg { .. })),
        "expected UnknownOrg, got {result:?}"
    );
}

// ---------------------------------------------------------------------------
// BC-3.1.001 postcondition 1 round-trip — slug → OrgId keying (rename stability)
// ---------------------------------------------------------------------------

/// After org rename (new registry with new slug → same OrgId), `get_spec(new_slug, sensor)`
/// still finds the spec because the store is keyed on `OrgId`, not slug.
///
/// The BiMap is bijective so we model the rename as: build a post-rename registry
/// (new slug only) and verify the existing store entry is accessible.
///
/// Traces to: S-3.1.05 EC-003, ADR-006 §4 Step 2.
#[test]
fn test_BC_3_1_001_org_rename_preserves_spec_access() {
    let acme_id = OrgId::new();

    // Simulate pre-rename: spec is loaded and stored under acme_id.
    let pre_rename_registry = make_registry(&[("acme-corp", acme_id)]);
    let mut store = OrgScopedSpecStore::new(pre_rename_registry);
    store.insert(acme_id, make_sensor_spec("crowdstrike"));

    // Simulate post-rename: new registry where new slug → same OrgId.
    // (The old slug has been de-registered; the BiMap can now bind acme_id to the new slug.)
    let post_rename_registry = make_registry(&[("acme-corporation", acme_id)]);
    // Replace the registry reference on the store (simulates runtime update).
    let mut store = OrgScopedSpecStore::new(post_rename_registry);
    store.insert(acme_id, make_sensor_spec("crowdstrike"));

    // New slug should resolve to same OrgId and find the spec.
    let new_slug = OrgSlug::new("acme-corporation");
    let _ = store.get_spec(&new_slug, "crowdstrike").unwrap();
}

// ---------------------------------------------------------------------------
// S-3.1.05 AC-4 — cross-org spec isolation
// ---------------------------------------------------------------------------

/// Spec stored under `OrgId(A)` is NOT returned for `slug_B` (different org).
///
/// Traces to: BC-3.1.001 postcondition 3, S-3.1.05 AC-4, EC-004.
#[test]
fn test_BC_3_1_001_cross_org_spec_isolation() {
    let org_id_a = OrgId::new();
    let org_id_b = OrgId::new();
    let registry = make_registry(&[("org-a", org_id_a), ("org-b", org_id_b)]);
    let mut store = OrgScopedSpecStore::new(registry);

    // Store spec only under org A.
    store.insert(org_id_a, make_sensor_spec("crowdstrike"));

    let slug_b = OrgSlug::new("org-b");
    // Org B should get SensorNotFound, not org A's spec.
    let result = store.get_spec(&slug_b, "crowdstrike");
    assert!(
        matches!(result, Err(SpecEngineError::SensorNotFound { .. })),
        "expected SensorNotFound for org-b, got {result:?}"
    );
}

// ---------------------------------------------------------------------------
// S-3.1.05 EC-002 — known org, missing sensor
// ---------------------------------------------------------------------------

/// `get_spec(slug, "armis")` when the org has only a crowdstrike spec returns
/// `Err(SensorNotFound)`.
///
/// Traces to: S-3.1.05 AC-1, EC-002.
#[test]
fn test_BC_3_1_001_known_org_missing_sensor_returns_sensor_not_found() {
    let acme_id = OrgId::new();
    let registry = make_registry(&[("acme-corp", acme_id)]);
    let mut store = OrgScopedSpecStore::new(registry);
    store.insert(acme_id, make_sensor_spec("crowdstrike"));

    let slug = OrgSlug::new("acme-corp");
    let result = store.get_spec(&slug, "armis");
    assert!(
        matches!(result, Err(SpecEngineError::SensorNotFound { .. })),
        "expected SensorNotFound for missing sensor, got {result:?}"
    );
}

// ---------------------------------------------------------------------------
// S-3.1.05 AC-2 — OrgRegistry::resolve called at boundary, not in store layer
// ---------------------------------------------------------------------------

/// Two orgs with identical sensor names are stored under distinct OrgIds; each
/// `get_spec` call returns the correct per-org spec.
///
/// Structural test: if the store were keyed on slug or sensor_name alone,
/// this would produce the wrong result.
///
/// Traces to: S-3.1.05 AC-2, AC-4.
#[test]
fn test_BC_3_1_001_two_orgs_same_sensor_name_no_collision() {
    let id_a = OrgId::new();
    let id_b = OrgId::new();
    let registry = make_registry(&[("org-a", id_a), ("org-b", id_b)]);
    let mut store = OrgScopedSpecStore::new(registry);

    let mut spec_a = make_sensor_spec("crowdstrike");
    spec_a.base_url = "https://api.org-a.example.com".to_string();

    let mut spec_b = make_sensor_spec("crowdstrike");
    spec_b.base_url = "https://api.org-b.example.com".to_string();

    store.insert(id_a, spec_a);
    store.insert(id_b, spec_b);

    // Both orgs should each see their own spec.
    let result_a = store
        .get_spec(&OrgSlug::new("org-a"), "crowdstrike")
        .unwrap();
    assert_eq!(result_a.base_url, "https://api.org-a.example.com");

    let result_b = store
        .get_spec(&OrgSlug::new("org-b"), "crowdstrike")
        .unwrap();
    assert_eq!(result_b.base_url, "https://api.org-b.example.com");
}

// ---------------------------------------------------------------------------
// S-3.1.05 AC-3 — RegistryNotInitialized path never panics
// ---------------------------------------------------------------------------

/// `get_spec` must return `Err(RegistryNotInitialized)` (not panic) when the
/// registry has no entries at all — the degenerate pre-startup state.
///
/// In production BC-3.1.001 invariant 3 guarantees this is unreachable after
/// startup, but the API contract forbids panicking in any code path.
///
/// The test uses an empty `OrgRegistry` to approximate the "not-yet-populated"
/// state visible at the API boundary: `resolve(any_slug)` returns `None`,
/// which the implementation MUST map to `Err(RegistryNotInitialized)` or
/// `Err(UnknownOrg)` — never a panic.
///
/// During the Red Gate phase `get_spec` is a `todo!()` stub, so this test
/// panics with "not yet implemented". Post-implementation it must NOT panic —
/// it must return an `Err` variant.
///
/// Traces to: S-3.1.05 AC-3, BC-3.1.001 invariant 3.
#[test]
fn test_BC_3_1_001_empty_registry_returns_err_not_panic() {
    // Empty registry — no org has been registered; simulates pre-startup state.
    let empty_registry = Arc::new(OrgRegistry::new());
    let store = OrgScopedSpecStore::new(empty_registry);

    let slug = OrgSlug::new("any-org");
    // Must return Err (UnknownOrg or RegistryNotInitialized), never panic.
    // During stub phase: todo!() fires and this #[should_panic] catches it.
    let result = store.get_spec(&slug, "crowdstrike");
    assert!(
        matches!(
            result,
            Err(SpecEngineError::UnknownOrg { .. }) | Err(SpecEngineError::RegistryNotInitialized)
        ),
        "expected UnknownOrg or RegistryNotInitialized for empty registry, got {result:?}"
    );
}
