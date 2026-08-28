//! Slug-isolation Red Gate Tests — RG-SLUG-001..004
//!
//! Traces to: S-ENGINE-LIMIT-EARLY-STOP-001 AC-010 (SLUG-001/002), AC-011 (SLUG-003/004)
//!            ADR-061 D2 (fail-closed when registry present + slug missing)
//!            ADR-061 D3 (synthetic slug when registry absent)
//!
//! # Purpose
//!
//! These tests verify that `resolve_source_refs` (ALL-scope path) and Step 3b
//! (bare-filter fan-out) correctly implement ADR-061 D2: when the `OrgRegistry`
//! is **present** but contains no slug mapping for an `OrgId`, the adapter is
//! SKIPPED and a `tracing::warn!` is emitted.  ADR-061 D3 governs the opposite
//! case — when the registry is **absent** (test/MVP mode), a synthetic slug is
//! used and the adapter IS called.
//!
//! ## RED / GREEN mechanics
//!
//! RED (current code — D2 not implemented):
//!   Both `resolve_source_refs` and Step 3b synthesize a slug for EVERY OrgId
//!   regardless of whether the registry is present or absent.  When the registry
//!   IS present but contains no slug for the OrgId, the target is still pushed
//!   and the adapter is called.  SLUG-001 and SLUG-003 assert `fetch_count == 0`;
//!   those assertions FAIL because `fetch_count == 1`.
//!
//! GREEN (post-D2 fix):
//!   `resolve_source_refs` (and Step 3b) check `org_registry.is_some()` inside
//!   the `None`-from-`slug_for` branch.  When the registry IS present, the OrgId
//!   is skipped; when absent, the synthetic slug is used.  SLUG-001 and SLUG-003
//!   PASS; SLUG-002 and SLUG-004 continue to PASS (regression sentinels).
//!
//! # Test Matrix
//!
//! | RG ID    | Path              | Registry state              | Assert          | State        |
//! |----------|-------------------|-----------------------------|-----------------|--------------|
//! | SLUG-001 | resolve_source_refs ALL-scope | present + slug missing | fetch_count==0  | RED gate     |
//! | SLUG-002 | resolve_source_refs ALL-scope | absent                 | fetch_count>=1  | regression   |
//! | SLUG-003 | Step 3b bare-filter           | present + slug missing | fetch_count==0  | RED gate     |
//! | SLUG-004 | Step 3b bare-filter           | absent                 | fetch_count>=1  | regression   |

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    non_snake_case,
    unused_imports
)]

use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use arrow::array::StringArray;
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use async_trait::async_trait;
use prism_core::{OrgId, OrgRegistry, OrgSlug, SensorId};
use prism_ocsf::OcsfNormalizer;
use prism_query::{
    engine::QueryOptions,
    materialization::{run_materialization_pipeline, MaterializationContext},
    memory::{build_session_context, MAX_MATERIALIZED_RECORDS, QUERY_MEMORY_POOL_BYTES},
};
use prism_sensors::{
    adapter::{FetchOutput, QueryParams, SensorAdapter, SensorError, SensorSpec},
    auth::SensorAuth,
    AdapterRegistry, CredentialResolver,
};

// ---------------------------------------------------------------------------
// SlugSpyAdapter — records fetch_count; sensor_type "spy"
// ---------------------------------------------------------------------------

/// Minimal adapter that increments `fetch_count` on every `fetch()` call.
///
/// Used to verify whether the materialization pipeline actually invokes the
/// adapter or skips it (e.g., because the OrgId has no registry mapping).
struct SlugSpyAdapter {
    /// Incremented inside `fetch()` — tests assert on this value.
    fetch_count: Arc<AtomicU64>,
}

impl std::fmt::Debug for SlugSpyAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SlugSpyAdapter")
            .field("fetch_count", &self.fetch_count.load(Ordering::Relaxed))
            .finish()
    }
}

#[async_trait]
impl SensorAdapter for SlugSpyAdapter {
    fn sensor_type(&self) -> SensorId {
        SensorId::from("spy")
    }

    fn sensor_name(&self) -> &'static str {
        "spy"
    }

    async fn fetch(
        &self,
        _spec: &SensorSpec,
        _params: &QueryParams,
        _auth: &dyn SensorAuth,
    ) -> Result<FetchOutput, SensorError> {
        self.fetch_count.fetch_add(1, Ordering::SeqCst);
        let schema = Arc::new(Schema::new(vec![Field::new(
            "spy_id",
            DataType::Utf8,
            false,
        )]));
        let arr = Arc::new(StringArray::from(vec!["spy-row-0"])) as _;
        let batch = RecordBatch::try_new(schema, vec![arr]).expect("spy batch must be valid");
        Ok(FetchOutput::new(vec![batch], false))
    }
}

// ---------------------------------------------------------------------------
// SlugStubCredentialResolver
// ---------------------------------------------------------------------------

struct SlugStubCredentialResolver;

impl CredentialResolver for SlugStubCredentialResolver {
    fn resolve(
        &self,
        _client_id: &str,
        _sensor_id: SensorId,
    ) -> Result<Box<dyn SensorAuth>, prism_sensors::SensorError> {
        struct StubAuth;
        impl SensorAuth for StubAuth {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
            fn auth_type_name(&self) -> &'static str {
                "custom_via_plugin"
            }
        }
        Ok(Box::new(StubAuth))
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a `MaterializationContext` with one `SlugSpyAdapter` registered under
/// `org_id`, and optionally an `OrgRegistry`.
///
/// Returns `(mat_ctx, fetch_count)` so tests can assert on the call count.
fn slug_mat_ctx(
    org_id: OrgId,
    org_registry: Option<Arc<OrgRegistry>>,
) -> (MaterializationContext, Arc<AtomicU64>) {
    let fetch_count = Arc::new(AtomicU64::new(0));
    let adapter = Arc::new(SlugSpyAdapter {
        fetch_count: Arc::clone(&fetch_count),
    });
    let mut registry = AdapterRegistry::new();
    registry.register(org_id, adapter);

    let mat_ctx = MaterializationContext::new_with_resolver(
        Arc::new(registry),
        Arc::new(OcsfNormalizer::new()),
        MAX_MATERIALIZED_RECORDS,
        Arc::new(SlugStubCredentialResolver),
        org_registry,
        None, // resolved_spec_map absent
    );
    (mat_ctx, fetch_count)
}

/// ALL-scope `QueryOptions` (no explicit client list).
fn all_scope_opts() -> QueryOptions {
    QueryOptions {
        clients: None,
        sensors: None,
        limit: Some(10),
        force_refresh: false,
        ..QueryOptions::default()
    }
}

// ===========================================================================
// SLUG-001 — RG-SLUG-001 (AC-010): resolve_source_refs, registry present +
//            slug missing → adapter SKIPPED, emit warn
// ===========================================================================

/// RG-SLUG-001 — ADR-061 D2 via `resolve_source_refs` ALL-scope path
///
/// Arrangement: `OrgRegistry` is **present** (non-None) but EMPTY — no slug
/// is registered for `org_id`.  The `spy_events` table fan-out goes through
/// the `resolve_source_refs` ALL-scope path.
///
/// ## RED / GREEN mechanics
///
/// RED (current code — D2 not implemented):
///   `resolve_source_refs` enters the `slug_for` = None else-branch and
///   synthesizes a slug regardless of whether the registry is present.
///   The target is pushed → adapter called → `fetch_count = 1`.
///   Assertion `fetch_count == 0` FAILS.
///
/// GREEN (post-D2 fix):
///   else-branch checks `org_registry.is_some()`.  Registry IS present →
///   OrgId has no mapping → target skipped → `fetch_count = 0` → PASSES.
///
/// ADR-061 D2: "when registry is present, treat absent slug as configuration
/// inconsistency: skip target and emit `query.org_slug_resolution_failure`."
#[tokio::test]
async fn test_rg_slug_001_resolve_source_refs_registry_present_slug_missing_skips_target_emits_warn(
) {
    let org_id = OrgId::new();
    // OrgRegistry is PRESENT but EMPTY — org_id has no mapping.
    let org_registry = Arc::new(OrgRegistry::new());
    let (mut mat_ctx, fetch_count) = slug_mat_ctx(org_id, Some(org_registry));
    let session_ctx = build_session_context(QUERY_MEMORY_POOL_BYTES).expect("session_ctx");

    // SELECT query against "spy_events" — sensor_id = "spy" is registered.
    let _result = run_materialization_pipeline(
        "SELECT * FROM spy_events LIMIT 10",
        &all_scope_opts(),
        &mut mat_ctx,
        &session_ctx,
    )
    .await;

    let fc = fetch_count.load(Ordering::SeqCst);
    assert_eq!(
        fc, 0,
        "RG-SLUG-001 (ADR-061 D2 — resolve_source_refs, registry present + slug missing): \
         adapter must be SKIPPED when OrgRegistry is present but contains no slug for the \
         OrgId (fetch_count={fc}). If non-zero, `resolve_source_refs` synthesized a slug \
         instead of skipping, violating D2 fail-closed dispatch."
    );
}

// ===========================================================================
// SLUG-002 — RG-SLUG-002 (AC-010): resolve_source_refs, registry ABSENT →
//            synthetic slug used (regression sentinel — must PASS before AND after)
// ===========================================================================

/// RG-SLUG-002 — ADR-061 D3 regression sentinel via `resolve_source_refs`
///
/// Arrangement: `OrgRegistry` is **absent** (None) — test/MVP mode.
/// Per ADR-061 D3, a synthetic slug derived from the `OrgId` is used and the
/// adapter IS called.
///
/// This test is a REGRESSION SENTINEL: it PASSES before AND after the D2 fix.
/// Its purpose is to ensure the fix does not accidentally disable the D3 path
/// (absent registry → still fan out with synthetic slug).
///
/// ADR-061 D3: "when registry is absent (test/MVP mode), synthesise a slug
/// from `OrgId` so the query can proceed."
#[tokio::test]
async fn test_rg_slug_002_resolve_source_refs_registry_absent_synthetic_slug_included() {
    let org_id = OrgId::new();
    // OrgRegistry is ABSENT — synthetic slug path (D3).
    let (mut mat_ctx, fetch_count) = slug_mat_ctx(org_id, None);
    let session_ctx = build_session_context(QUERY_MEMORY_POOL_BYTES).expect("session_ctx");

    let _result = run_materialization_pipeline(
        "SELECT * FROM spy_events LIMIT 10",
        &all_scope_opts(),
        &mut mat_ctx,
        &session_ctx,
    )
    .await;

    let fc = fetch_count.load(Ordering::SeqCst);
    assert!(
        fc >= 1,
        "RG-SLUG-002 (ADR-061 D3 regression — resolve_source_refs, registry absent): \
         adapter must be called when OrgRegistry is absent (D3 synthetic slug path). \
         fetch_count={fc}. A count of 0 means the D2 fix incorrectly disabled D3."
    );
}

// ===========================================================================
// SLUG-003 — RG-SLUG-003 (AC-011): Step 3b bare-filter, registry present +
//            slug missing → adapter SKIPPED, emit warn
// ===========================================================================

/// RG-SLUG-003 — ADR-061 D2 via Step 3b bare-filter fan-out path
///
/// Arrangement: `OrgRegistry` is **present** but EMPTY.  The bare-filter query
/// `"status = 'active'"` (no explicit source) routes through Step 3b which
/// enumerates ALL registered adapters and synthesises FanOutTargets.
///
/// ## RED / GREEN mechanics
///
/// RED (current code — D2 not implemented in Step 3b):
///   Step 3b at line ~801 of `materialization.rs` always synthesises a slug:
///   `OrgSlug::new(format!("org-{}", &org_id.to_string()[..8]))`.
///   The `mat_ctx.org_registry` field is NEVER consulted.
///   Target is pushed → adapter called → `fetch_count = 1`.
///   Assertion `fetch_count == 0` FAILS.
///
/// GREEN (post-D2 fix for Step 3b):
///   Step 3b checks `mat_ctx.org_registry.as_ref().and_then(|reg| reg.slug_for(&org_id))`.
///   Registry IS present but slug absent → target SKIPPED → `fetch_count = 0` → PASSES.
///
/// ADR-061 D2 applies to BOTH `resolve_source_refs` AND Step 3b (the story groups
/// them under AC-011 as the Step 3b variant of the same D2 behaviour).
#[tokio::test]
async fn test_rg_slug_003_bare_filter_step3b_registry_present_slug_missing_skips_target_emits_warn()
{
    let org_id = OrgId::new();
    // OrgRegistry is PRESENT but EMPTY — org_id has no mapping.
    let org_registry = Arc::new(OrgRegistry::new());
    let (mut mat_ctx, fetch_count) = slug_mat_ctx(org_id, Some(org_registry));
    let session_ctx = build_session_context(QUERY_MEMORY_POOL_BYTES).expect("session_ctx");

    // Bare-filter query — no explicit source, triggers Step 3b fan-out.
    let _result = run_materialization_pipeline(
        "status = 'active'",
        &all_scope_opts(),
        &mut mat_ctx,
        &session_ctx,
    )
    .await;

    let fc = fetch_count.load(Ordering::SeqCst);
    assert_eq!(
        fc, 0,
        "RG-SLUG-003 (ADR-061 D2 — Step 3b bare-filter, registry present + slug missing): \
         adapter must be SKIPPED when OrgRegistry is present but contains no slug for the \
         OrgId (fetch_count={fc}). If non-zero, Step 3b synthesised a slug without consulting \
         `mat_ctx.org_registry`, violating D2 fail-closed dispatch for bare-filter queries."
    );
}

// ===========================================================================
// SLUG-004 — RG-SLUG-004 (AC-011): Step 3b bare-filter, registry ABSENT →
//            synthetic slug (regression sentinel — must PASS before AND after)
// ===========================================================================

/// RG-SLUG-004 — ADR-061 D3 regression sentinel via Step 3b bare-filter
///
/// Arrangement: `OrgRegistry` is **absent** (None).  Per ADR-061 D3, Step 3b
/// falls back to a synthetic slug and the adapter IS called.
///
/// This test is a REGRESSION SENTINEL: it PASSES before AND after the D2 fix.
/// Its purpose is to ensure the Step 3b D2 fix does not accidentally disable
/// the D3 path (absent registry → still fan out with synthetic slug).
///
/// ADR-061 D3: bare-filter + absent registry → synthetic slug → adapter called.
#[tokio::test]
async fn test_rg_slug_004_bare_filter_step3b_registry_absent_synthetic_slug_included() {
    let org_id = OrgId::new();
    // OrgRegistry is ABSENT — synthetic slug path (D3).
    let (mut mat_ctx, fetch_count) = slug_mat_ctx(org_id, None);
    let session_ctx = build_session_context(QUERY_MEMORY_POOL_BYTES).expect("session_ctx");

    // Bare-filter query — no explicit source, triggers Step 3b fan-out.
    let _result = run_materialization_pipeline(
        "status = 'active'",
        &all_scope_opts(),
        &mut mat_ctx,
        &session_ctx,
    )
    .await;

    let fc = fetch_count.load(Ordering::SeqCst);
    assert!(
        fc >= 1,
        "RG-SLUG-004 (ADR-061 D3 regression — Step 3b bare-filter, registry absent): \
         adapter must be called when OrgRegistry is absent (D3 synthetic slug path). \
         fetch_count={fc}. A count of 0 means the D2 fix incorrectly disabled D3 for \
         the bare-filter path."
    );
}
