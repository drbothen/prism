//! HIGH-006 (S-PLUGIN-PREREQ-C): Property test verifying that OrgSlug::new()
//! succeeds for any UUID-derived prefix.
//!
//! The production fallback in prism-query/src/materialization.rs constructs a
//! synthetic OrgSlug from `format!("org-{}", &org_id.to_string()[..8])`. This
//! test verifies that pattern ALWAYS produces a valid OrgSlug regardless of
//! OrgId's Display format, locking the invariant against transitive drift.

use prism_core::{OrgId, OrgSlug};

/// Verify that the synthetic slug pattern always produces a valid OrgSlug.
///
/// UUID v7 Display is `xxxxxxxx-xxxx-7xxx-yxxx-xxxxxxxxxxxx` (hyphenated lowercase).
/// The first 8 chars are always timestamp hex: `[0-9a-f]{8}`.
/// Prefixed with "org-", the result is `org-[0-9a-f]{8}` = 12 chars, all valid.
#[test]
fn test_org_slug_from_uuid_prefix_always_valid() {
    // Generate 100 fresh OrgIds and verify the synthetic slug pattern is always valid.
    for _ in 0..100 {
        let id = OrgId::new();
        let candidate = format!("org-{}", &id.to_string()[..8]);
        let slug = OrgSlug::new(&candidate);
        assert!(
            slug.is_ok(),
            "synthetic slug '{candidate}' from OrgId '{id}' must be valid for OrgSlug"
        );
    }
}

/// Verify the fallback sentinel is also always valid.
#[test]
fn test_synthetic_unmapped_sentinel_is_valid() {
    let sentinel = OrgSlug::new("synthetic-unmapped");
    assert!(
        sentinel.is_ok(),
        "'synthetic-unmapped' must always be a valid OrgSlug"
    );
}
