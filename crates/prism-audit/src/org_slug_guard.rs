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
/// ## TODO (implementer)
///
/// This stub ALWAYS returns `OrgNotInRegistry` so that all SEC-007 tests fail
/// with assertion errors at the Red Gate. Replace this body with the real
/// implementation that:
/// 1. Calls `registry.slug_for(&entry.org_id)`.
/// 2. Matches on the result (NEVER calls unwrap()).
/// 3. Returns the appropriate `SlugCheckResult` variant.
/// 4. Emits `tracing::warn!` for `Mismatched` and `OrgNotInRegistry` cases.
pub fn validate_org_slug_cross_check(
    registry: &OrgRegistry,
    entry: &AuditEntry,
) -> SlugCheckResult {
    // TODO(implementer): replace this stub with the real cross-check.
    // Returning OrgNotInRegistry causes SEC-007 assertion tests to FAIL at Red Gate.
    let _ = (registry, entry); // suppress unused-variable warnings in stub
    SlugCheckResult::OrgNotInRegistry
}
