//! Org-slug cross-check guard for audit record construction.
//!
//! Provides `validate_org_slug_cross_check` and `SlugCheckResult` to verify
//! that the `org_slug` in an `AuditEntry` is consistent with what
//! `OrgRegistry::slug_for(org_id)` returns at emit time.
//!
//! ## Contract (BC-3.1.002 postcondition / AC-006)
//!
//! - When `OrgRegistry::slug_for(org_id)` returns `Some(slug)` that equals
//!   `entry.org_slug`, return `SlugCheckResult::Matched`.
//! - When the slug differs, return `SlugCheckResult::Mismatched { registry_slug }`
//!   and emit `tracing::warn!` (NOT panic).
//! - When `slug_for` returns `None` (EC-007), return `SlugCheckResult::OrgNotInRegistry`
//!   and emit `tracing::warn!` (NOT panic).
//!
//! ## TODO (implementer — W3-FIX-CODE-002 AC-006)
//!
//! 1. Implement `validate_org_slug_cross_check` to call `registry.slug_for(&entry.org_id)`
//!    and return the appropriate `SlugCheckResult` variant.
//! 2. Emit `tracing::warn!` for `Mismatched` and `OrgNotInRegistry` cases.
//! 3. Wire `validate_org_slug_cross_check` into `AuditEmitterService::call()` after
//!    `AuditEntry::new()` is constructed and BEFORE `emit()` is called.
//! 4. Do NOT abort audit emission on any variant (audit-must-not-fail semantics;
//!    BC-3.1.002 postcondition).
//!
//! ## Red Gate
//!
//! This stub returns `OrgNotInRegistry` for ALL inputs, causing the SEC-007 tests
//! to fail with assertion errors:
//! - `test_BC_3_1_002_SEC007_slug_matches_registry_no_warning_emitted` →
//!   expects `Matched` but gets `OrgNotInRegistry`.
//! - `test_BC_3_1_002_SEC007_audit_entry_emitted_when_slug_matches` →
//!   same failure.
//! - `test_BC_3_1_002_SEC007_mismatched_slug_emits_warn_not_panic` →
//!   expects `Mismatched { .. }` but gets `OrgNotInRegistry`.
//! - `test_BC_3_1_002_SEC007_audit_entry_emitted_even_on_slug_mismatch` →
//!   same.

use crate::audit_entry::AuditEntry;
use prism_core::org_registry::OrgRegistry;
use prism_core::tenant::OrgSlug;

/// Result of cross-checking an `AuditEntry`'s `org_slug` against `OrgRegistry`.
///
/// Returned by `validate_org_slug_cross_check`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlugCheckResult {
    /// The registry has a matching entry and the slug is consistent.
    Matched,
    /// The registry has an entry for this org but the slug differs.
    ///
    /// The production code must emit `tracing::warn!` for this case (AC-006).
    Mismatched {
        /// The slug the registry holds for this org_id.
        registry_slug: OrgSlug,
    },
    /// The registry has no entry for this org_id (EC-007).
    ///
    /// The production code must emit `tracing::warn!` for this case (EC-007).
    OrgNotInRegistry,
}

/// Cross-check `entry.org_slug` against `OrgRegistry::slug_for(entry.org_id)`.
///
/// ## Contract (BC-3.1.002 postcondition / AC-006)
///
/// - `Matched`: registry holds a slug for this org_id that equals `entry.org_slug`.
/// - `Mismatched { registry_slug }`: registry holds a slug but it differs from
///   `entry.org_slug`. Emits `tracing::warn!` — does NOT abort audit emission.
/// - `OrgNotInRegistry`: registry has no entry for this org_id (EC-007). Emits
///   `tracing::warn!` — does NOT abort audit emission.
///
/// NEVER calls `unwrap()` on the result of `slug_for` (AC-006 architecture compliance).
pub fn validate_org_slug_cross_check(
    registry: &OrgRegistry,
    entry: &AuditEntry,
) -> SlugCheckResult {
    match registry.slug_for(&entry.org_id) {
        Some(registry_slug) if registry_slug == entry.org_slug => SlugCheckResult::Matched,
        Some(registry_slug) => {
            tracing::warn!(
                "audit org_slug mismatch: entry={}, registry={}",
                entry.org_slug.as_str(),
                registry_slug.as_str(),
            );
            SlugCheckResult::Mismatched { registry_slug }
        }
        None => {
            tracing::warn!(
                "audit org_slug cross-check: org_id={} not in OrgRegistry (EC-007)",
                entry.org_id,
            );
            SlugCheckResult::OrgNotInRegistry
        }
    }
}
