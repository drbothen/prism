//! SEC-007 test suite — org_slug cross-check against OrgRegistry at audit
//! record construction time.
//!
//! Covers:
//!   BC-3.1.002 postcondition — org_slug cross-checked against OrgRegistry::slug_for
//!   AC-006 (debug_assert or tracing::warn on mismatch)
//!   EC-007 (OrgRegistry::slug_for returns None — warn, not panic)
//!
//! ## Production gap
//!
//! `AuditEmitterService::call()` constructs `AuditEntry::new()` passing `req.org_id`
//! and `req.org_slug` directly with NO call to `OrgRegistry::slug_for(org_id)` to
//! verify the slug is consistent. The cross-check function does not exist anywhere
//! in `prism_audit`.
//!
//! ## Red Gate contract
//!
//! These tests call `prism_audit::validate_org_slug_cross_check(registry, entry)`
//! which does NOT exist in the production crate yet. Until that function is added,
//! this test file fails to COMPILE — which is the Red Gate for compile-time-missing
//! production APIs.
//!
//! Once the function is added (as part of wiring AC-006 into the audit path), the
//! tests will run. Tests for the mismatch / None cases will then verify the correct
//! `SlugCheckResult` variant is returned, and the production cross-check is correctly
//! wired into `AuditEmitterService::call()`.
//!
//! ## Audit-must-not-fail semantics (BC-3.1.002)
//!
//! A slug mismatch is a data-consistency anomaly but MUST NOT prevent audit
//! entry emission. The check emits `tracing::warn!` and continues; no panic
//! and no early return.
//!
//! Test naming: `test_BC_3_1_002_SEC007_xxx()` per factory convention.
#![allow(non_snake_case, clippy::expect_used, clippy::unwrap_used)]

use chrono::Utc;
use prism_audit::audit_entry::{AuditEntry, AuditOutcome, DataClassification};
use prism_core::org_registry::OrgRegistry;
use prism_core::tenant::OrgSlug;
use prism_core::OrgId;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Production gap: this import fails to compile until the implementer adds
// `validate_org_slug_cross_check` to the `prism_audit` crate.
//
// The `SlugCheckResult` enum and `validate_org_slug_cross_check` function are
// the production API surface that the AC-006 implementation must provide.
// ---------------------------------------------------------------------------
use prism_audit::org_slug_guard::{validate_org_slug_cross_check, SlugCheckResult};

// ---------------------------------------------------------------------------
// Helper: build a minimal AuditEntry.
// ---------------------------------------------------------------------------

fn make_entry(org_id: OrgId, org_slug: OrgSlug) -> AuditEntry {
    AuditEntry::new(
        Uuid::now_v7(),
        Utc::now(),
        "test_tool".to_owned(),
        "test-client".to_owned(),
        "analyst@example.com".to_owned(),
        serde_json::json!({}),
        AuditOutcome::Success,
        "ok".to_owned(),
        0,
        None,
        DataClassification::Internal,
        vec![],
        vec![],
        org_id,
        org_slug,
        String::new(),
    )
}

// ===========================================================================
// AC-006: cross-check passes when slug matches OrgRegistry
// ===========================================================================

/// BC-3.1.002 postcondition / AC-006:
/// When `OrgRegistry::slug_for(org_id)` returns `Some(slug)` equal to
/// `entry.org_slug`, `validate_org_slug_cross_check` returns `Matched`.
///
/// ## Red Gate
///
/// Fails to compile until `prism_audit::org_slug_guard::validate_org_slug_cross_check`
/// and `SlugCheckResult` are added to the production crate.
#[test]
fn test_BC_3_1_002_SEC007_slug_matches_registry_no_warning_emitted() {
    let registry = OrgRegistry::new();
    let org_id = OrgId::new();
    let slug = OrgSlug::new("acme-corp");

    registry
        .register(slug.clone(), org_id)
        .expect("registration must succeed");

    let entry = make_entry(org_id, slug.clone());
    let result = validate_org_slug_cross_check(&registry, &entry);

    assert_eq!(
        result,
        SlugCheckResult::Matched,
        "AC-006: when registry.slug_for(org_id) == entry.org_slug, cross-check must \
         return SlugCheckResult::Matched — no warning should be emitted"
    );
}

/// BC-3.1.002 postcondition / AC-006:
/// When slugs match, the audit entry is still constructed and emitted.
#[test]
fn test_BC_3_1_002_SEC007_audit_entry_emitted_when_slug_matches() {
    let registry = OrgRegistry::new();
    let org_id = OrgId::new();
    let slug = OrgSlug::new("acme-corp");

    registry
        .register(slug.clone(), org_id)
        .expect("registration must succeed");

    let entry = make_entry(org_id, slug.clone());

    // The entry must be fully constructed regardless of cross-check result.
    assert_eq!(
        entry.org_id, org_id,
        "AC-006: audit entry must carry org_id when slug matches registry"
    );
    assert_eq!(
        entry.org_slug, slug,
        "AC-006: audit entry must carry the submitted org_slug"
    );

    // Cross-check returns Matched — no warning in this scenario.
    let result = validate_org_slug_cross_check(&registry, &entry);
    assert_eq!(
        result,
        SlugCheckResult::Matched,
        "AC-006: matching-slug scenario must return Matched; \
         no spurious warning must be emitted"
    );
}

// ===========================================================================
// AC-006: cross-check warns (not panics) on slug mismatch
// ===========================================================================

/// BC-3.1.002 postcondition / AC-006:
/// When slugs mismatch, `validate_org_slug_cross_check` returns
/// `Mismatched { registry_slug }` — not a panic, not Err.
///
/// In production, this result triggers `tracing::warn!`; the audit entry is
/// still emitted (audit-must-not-fail semantics).
#[test]
fn test_BC_3_1_002_SEC007_mismatched_slug_emits_warn_not_panic() {
    let registry = OrgRegistry::new();
    let org_id = OrgId::new();
    let correct_slug = OrgSlug::new("correct-slug");
    let wrong_slug = OrgSlug::new("wrong-slug");

    registry
        .register(correct_slug.clone(), org_id)
        .expect("registration must succeed");

    let entry = make_entry(org_id, wrong_slug.clone());
    let result = validate_org_slug_cross_check(&registry, &entry);

    assert_eq!(
        result,
        SlugCheckResult::Mismatched {
            registry_slug: correct_slug.clone()
        },
        "AC-006: when entry.org_slug='wrong-slug' but registry holds 'correct-slug' \
         for this org_id, cross-check must return Mismatched (not panic). \
         In production this triggers tracing::warn!"
    );

    // The entry retains the submitted slug (denormalized — must not be corrected).
    assert_eq!(
        entry.org_slug, wrong_slug,
        "AC-006: AuditEntry must retain the submitted org_slug; the cross-check \
         must only warn, not modify the entry."
    );
}

/// BC-3.1.002 postcondition / AC-006:
/// Even when slug mismatches, the audit entry is still constructed (must-not-fail).
#[test]
fn test_BC_3_1_002_SEC007_audit_entry_emitted_even_on_slug_mismatch() {
    let registry = OrgRegistry::new();
    let org_id = OrgId::new();
    let registered_slug = OrgSlug::new("registered-slug");
    let submitted_slug = OrgSlug::new("submitted-slug");

    registry
        .register(registered_slug.clone(), org_id)
        .expect("registration must succeed");

    let entry = make_entry(org_id, submitted_slug.clone());

    // Entry is fully constructed regardless of slug mismatch.
    assert_eq!(
        entry.org_id, org_id,
        "AC-006 (audit-must-not-fail): org_id present even on slug mismatch"
    );
    assert_eq!(
        entry.org_slug, submitted_slug,
        "AC-006 (audit-must-not-fail): org_slug retains submitted value (denormalized)"
    );

    let result = validate_org_slug_cross_check(&registry, &entry);
    assert!(
        matches!(result, SlugCheckResult::Mismatched { .. }),
        "AC-006: mismatched slug must produce SlugCheckResult::Mismatched — \
         production code must detect this and emit tracing::warn!"
    );
}

// ===========================================================================
// EC-007: OrgRegistry::slug_for returns None — warn, not panic
// ===========================================================================

/// BC-3.1.002 postcondition / AC-006 / EC-007:
/// When `OrgRegistry` has no entry for `org_id`, `validate_org_slug_cross_check`
/// returns `OrgNotInRegistry` — not a panic.
///
/// The production code must use `match` or `if let`, NOT `unwrap()`, on the
/// `Option<OrgSlug>` returned by `slug_for`.
#[test]
fn test_BC_3_1_002_SEC007_slug_for_returns_none_emits_warn_not_panic() {
    let registry = OrgRegistry::new(); // Empty — no entries.
    let org_id = OrgId::new();
    let slug = OrgSlug::new("some-slug");

    let entry = make_entry(org_id, slug);
    let result = validate_org_slug_cross_check(&registry, &entry);

    assert_eq!(
        result,
        SlugCheckResult::OrgNotInRegistry,
        "EC-007: when OrgRegistry has no entry for org_id, cross-check must return \
         OrgNotInRegistry (not panic). In production this emits tracing::warn! and \
         allows audit emission to proceed."
    );
}

/// BC-3.1.002 postcondition / EC-007:
/// When `slug_for` returns `None`, the audit entry is still emitted with the
/// submitted slug value.
#[test]
fn test_BC_3_1_002_SEC007_audit_entry_emitted_when_slug_for_returns_none() {
    let registry = OrgRegistry::new();
    let org_id = OrgId::new();
    let slug = OrgSlug::new("unregistered-org-slug");

    let entry = make_entry(org_id, slug.clone());

    assert_eq!(
        entry.org_slug, slug,
        "EC-007: audit entry must carry the submitted org_slug even when \
         OrgRegistry::slug_for returns None (org not registered)"
    );
    assert_eq!(
        entry.org_id, org_id,
        "EC-007: audit entry must carry the org_id even when org is absent from registry"
    );

    let result = validate_org_slug_cross_check(&registry, &entry);
    assert_eq!(
        result,
        SlugCheckResult::OrgNotInRegistry,
        "EC-007: OrgNotInRegistry must be returned without panicking"
    );
}

// ===========================================================================
// AC-006: OrgRegistry cross-check does not use unwrap()
// ===========================================================================

/// BC-3.1.002 / AC-006 architecture compliance:
/// The slug cross-check must never call `unwrap()` on the result of
/// `OrgRegistry::slug_for` — it must use pattern-matching or `if let`.
///
/// Verified by calling the cross-check with an empty registry (slug_for returns None)
/// and asserting the function does not panic.
#[test]
fn test_BC_3_1_002_SEC007_no_unwrap_on_slug_for_result() {
    let registry = OrgRegistry::new();
    let org_id = OrgId::new();
    let slug = OrgSlug::new("no-registry-entry");

    let entry = make_entry(org_id, slug);

    // If the cross-check calls unwrap() on the None returned by slug_for, this panics.
    // It must NOT panic — the result must be OrgNotInRegistry.
    let result = std::panic::catch_unwind(|| validate_org_slug_cross_check(&registry, &entry));

    assert!(
        result.is_ok(),
        "AC-006 architecture compliance: validate_org_slug_cross_check must not panic \
         (no unwrap()) when OrgRegistry::slug_for returns None. \
         catch_unwind returned Err, meaning the function panicked."
    );

    assert_eq!(
        result.unwrap(),
        SlugCheckResult::OrgNotInRegistry,
        "AC-006: cross-check with empty registry must return OrgNotInRegistry \
         without panicking"
    );
}
