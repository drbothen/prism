//! Red Gate tests for S-3.1.03 — OrgRegistry bijective BiMap.
//!
//! # BC Anchors
//!
//! | BC ID       | Title                                                  |
//! |-------------|--------------------------------------------------------|
//! | BC-3.1.001  | OrgRegistry Bijective Slug/UUID Resolution             |
//! | BC-3.1.003  | OrgRegistry Maintains Strict Bijectivity at All Times  |
//! | BC-3.1.004  | OrgRegistry Rejects Duplicate Slugs and UUIDs          |
//!
//! # Test Inventory
//!
//! All tests below MUST FAIL before implementation (every method body is `todo!()`).
//!
//! | Test function                                                  | AC / EC / TV            | Fails because                |
//! |----------------------------------------------------------------|-------------------------|------------------------------|
//! | test_BC_3_1_001_tv_01_resolve_known_slug                       | TV-3.1.001-01           | todo!() in register/resolve  |
//! | test_BC_3_1_001_tv_02_resolve_unknown_slug_returns_none        | TV-3.1.001-02           | todo!() in resolve           |
//! | test_BC_3_1_001_tv_03_slug_for_known_id                        | TV-3.1.001-03           | todo!() in register/slug_for |
//! | test_BC_3_1_001_tv_04_slug_for_unknown_id_returns_none         | TV-3.1.001-04           | todo!() in slug_for          |
//! | test_BC_3_1_001_tv_05_round_trip_consistency                   | TV-3.1.001-05 / AC-1    | todo!() in all methods       |
//! | test_BC_3_1_001_ac2_resolve_unknown_no_side_effect             | AC-2                    | todo!() in resolve/len       |
//! | test_BC_3_1_001_ec001_resolve_before_any_register_returns_none | EC-001                  | todo!() in resolve           |
//! | test_BC_3_1_001_ec002_slug_for_unregistered_id_returns_none    | EC-002                  | todo!() in slug_for          |
//! | test_BC_3_1_001_ec004_max_length_slug_registers_and_resolves   | EC-004                  | todo!() in register/resolve  |
//! | test_BC_3_1_001_invariant_bimap_equivalence                    | BC-3.1.001 invariant 2  | todo!() in all methods       |
//! | test_BC_3_1_003_tv_01_basic_bijectivity                        | TV-3.1.003-01           | todo!() in all methods       |
//! | test_BC_3_1_003_tv_02_slug_conflict_rejected                   | TV-3.1.003-02           | todo!() in register          |
//! | test_BC_3_1_003_tv_03_id_conflict_rejected                     | TV-3.1.003-03           | todo!() in register          |
//! | test_BC_3_1_003_ec001_slug_conflict_preserves_registry         | BC-3.1.003 EC-001       | todo!() in register          |
//! | test_BC_3_1_003_ec002_id_conflict_preserves_registry           | BC-3.1.003 EC-002       | todo!() in register          |
//! | test_BC_3_1_003_ec004_single_entry_bijection_holds             | BC-3.1.003 EC-004       | todo!() in all methods       |
//! | test_BC_3_1_003_ec005_100_entries_forward_eq_reverse_len       | BC-3.1.003 EC-005       | todo!() in all methods       |
//! | test_BC_3_1_003_invariant_forward_len_eq_reverse_len           | BC-3.1.003 invariant 3  | todo!() in all methods       |
//! | test_BC_3_1_003_proptest_bijection_size_invariant              | VP-3.1.003-01 / AC-5    | todo!() in all methods       |
//! | test_BC_3_1_004_tv_01_slug_conflict_error_fields               | TV-3.1.004-01 / AC-6    | todo!() in register          |
//! | test_BC_3_1_004_tv_02_id_conflict_error_fields                 | TV-3.1.004-02 / AC-7    | todo!() in register          |
//! | test_BC_3_1_004_tv_03_no_partial_state_after_rejection         | TV-3.1.004-03           | todo!() in register          |
//! | test_BC_3_1_004_ac8_exact_duplicate_is_idempotent              | AC-8 / EC-003           | todo!() in register          |
//! | test_BC_3_1_004_ec001_slug_conflict_is_err_slug_conflict       | BC-3.1.004 EC-001       | todo!() in register          |
//! | test_BC_3_1_004_ec002_id_conflict_is_err_id_conflict           | BC-3.1.004 EC-002       | todo!() in register          |
//! | test_BC_3_1_004_ec005_valid_after_rejected_attempt             | BC-3.1.004 EC-005       | todo!() in register          |
//! | test_BC_3_1_004_invariant1_register_atomic_no_partial_state    | BC-3.1.004 invariant 1  | todo!() in register          |
//! | test_BC_3_1_004_invariant2_no_silent_overwrite                 | BC-3.1.004 invariant 2  | todo!() in register          |
//! | test_BC_3_1_004_invariant3_slug_conflict_error_display         | BC-3.1.004 invariant 3  | todo!() in register          |
//! | test_BC_3_1_004_invariant3_id_conflict_error_display           | BC-3.1.004 invariant 3  | todo!() in register          |
//! | test_BC_3_1_004_proptest_size_unchanged_on_error               | VP-3.1.004-01           | todo!() in register          |
//! | test_BC_3_1_004_proptest_successful_resolve_after_rejection    | VP-3.1.004-04           | todo!() in register          |
//! | test_BC_3_1_004_is_empty_new_registry                         | BC-3.1.001              | todo!() in new/is_empty      |
//! | test_BC_3_1_004_len_reflects_registrations                    | BC-3.1.001              | todo!() in register/len      |
//! | test_BC_3_1_001_ec005_concurrent_reads_are_safe                | EC-005 / BC-3.1.004 CC  | todo!() in all methods       |

// ── Imports ─────────────────────────────────────────────────────────────────

use std::sync::Arc;
use std::thread;

use prism_core::ids::OrgId;
use prism_core::org_registry::RegistrationError;
use prism_core::tenant::OrgSlug;
use prism_core::OrgRegistry;
use proptest::prelude::*;
use uuid::Uuid;

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Construct a valid `OrgSlug` from a string literal. Panics if invalid.
fn slug(s: &str) -> OrgSlug {
    OrgSlug::new(s).unwrap()
}

/// Construct a fresh `OrgId` from a newly minted UUID v7.
fn org_id() -> OrgId {
    OrgId::new()
}

/// Build a known `OrgId` from a fixed v7 UUID for deterministic test vectors.
fn org_id_from_v7_bytes(bytes: [u8; 16]) -> OrgId {
    let u = Uuid::from_bytes(bytes);
    OrgId::from_uuid(u)
}

/// A canonical UUID v7 for TV use — version nibble (byte 6 top nibble) = 0x7_.
const UUID_A_BYTES: [u8; 16] = [
    0x01, 0x90, 0x56, 0x78, // time_high (v7 ms timestamp)
    0x9a, 0xbc, // time_mid
    0x7d, 0xef, // ver=7, rand_a
    0x80, 0x12, // variant=10, rand_b
    0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, // node
];

const UUID_B_BYTES: [u8; 16] = [
    0x01, 0x90, 0x56, 0x78, 0x9a, 0xbc, 0x7e, 0xf0, 0x80, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xdf,
];

// ═══════════════════════════════════════════════════════════════════════════════
// BC-3.1.001 — OrgRegistry Bijective Slug/UUID Resolution
// ═══════════════════════════════════════════════════════════════════════════════

/// TV-3.1.001-01: resolve("acme-corp") returns Some(OrgId(uuid-A)) after registration.
///
/// BC-3.1.001 postcondition 1: forward resolution returns the registered OrgId.
#[test]
fn test_BC_3_1_001_tv_01_resolve_known_slug() {
    let registry = OrgRegistry::new();
    let uuid_a = org_id_from_v7_bytes(UUID_A_BYTES);
    let s = slug("acme-corp");
    registry
        .register(s.clone(), uuid_a)
        .expect("registration must succeed");
    let resolved = registry.resolve(&s);
    assert_eq!(
        resolved,
        Some(uuid_a),
        "TV-3.1.001-01: resolve('acme-corp') must return Some(uuid-A)"
    );
}

/// TV-3.1.001-02: resolve("unknown-org") returns None for a slug not in registry.
///
/// BC-3.1.001 postcondition 2: missing slug returns None with no side effect.
#[test]
fn test_BC_3_1_001_tv_02_resolve_unknown_slug_returns_none() {
    let registry = OrgRegistry::new();
    let result = registry.resolve(&slug("unknown-org"));
    assert_eq!(
        result, None,
        "TV-3.1.001-02: resolve of unknown slug must return None"
    );
}

/// TV-3.1.001-03: slug_for(OrgId(uuid-A)) returns Some(OrgSlug("acme-corp")).
///
/// BC-3.1.001 postcondition 1: reverse resolution returns the registered slug.
#[test]
fn test_BC_3_1_001_tv_03_slug_for_known_id() {
    let registry = OrgRegistry::new();
    let uuid_a = org_id_from_v7_bytes(UUID_A_BYTES);
    let s = slug("acme-corp");
    registry
        .register(s.clone(), uuid_a)
        .expect("registration must succeed");
    let result = registry.slug_for(&uuid_a);
    assert_eq!(
        result,
        Some(s),
        "TV-3.1.001-03: slug_for(uuid-A) must return Some('acme-corp')"
    );
}

/// TV-3.1.001-04: slug_for(unknown OrgId) returns None.
///
/// BC-3.1.001 postcondition 4: missing OrgId returns None; no log, no side effect.
#[test]
fn test_BC_3_1_001_tv_04_slug_for_unknown_id_returns_none() {
    let registry = OrgRegistry::new();
    let unknown = org_id();
    let result = registry.slug_for(&unknown);
    assert_eq!(
        result, None,
        "TV-3.1.001-04: slug_for of unknown OrgId must return None"
    );
}

/// TV-3.1.001-05: Round-trip consistency — resolve then slug_for returns original slug.
///
/// BC-3.1.001 postcondition 1: `resolve(slug).and_then(|id| slug_for(id)) == Some(slug)`.
/// AC-1 (traces to BC-3.1.001 postcondition 1).
#[test]
fn test_BC_3_1_001_tv_05_round_trip_consistency() {
    let registry = OrgRegistry::new();
    let uuid_a = org_id_from_v7_bytes(UUID_A_BYTES);
    let s = slug("acme-corp");
    registry
        .register(s.clone(), uuid_a)
        .expect("registration must succeed");

    let resolved_id = registry
        .resolve(&s)
        .expect("slug must resolve after registration");
    let recovered_slug = registry
        .slug_for(&resolved_id)
        .expect("id must resolve after registration");

    assert_eq!(
        recovered_slug, s,
        "TV-3.1.001-05: round-trip resolve(slug) -> slug_for(id) must recover original slug"
    );
}

/// AC-2: resolve of unknown slug returns None AND registry size is unchanged.
///
/// BC-3.1.001 postcondition 2: "resolve(unknown_slug) returns None without modifying registry state."
#[test]
fn test_BC_3_1_001_ac2_resolve_unknown_no_side_effect() {
    let registry = OrgRegistry::new();
    let uuid_a = org_id_from_v7_bytes(UUID_A_BYTES);
    registry
        .register(slug("acme-corp"), uuid_a)
        .expect("initial registration must succeed");

    let size_before = registry.len();
    let result = registry.resolve(&slug("nonexistent-org"));
    let size_after = registry.len();

    assert_eq!(
        result, None,
        "AC-2: resolve of unknown slug must return None"
    );
    assert_eq!(
        size_before, size_after,
        "AC-2: resolve must not modify registry size (no side effect)"
    );
}

/// EC-001: resolve called before any register returns None; no panic.
///
/// BC-3.1.001 edge case 1.
#[test]
fn test_BC_3_1_001_ec001_resolve_before_any_register_returns_none() {
    let registry = OrgRegistry::new();
    // No registrations — registry is empty.
    let result = registry.resolve(&slug("any-slug"));
    assert_eq!(
        result, None,
        "EC-001: resolve on empty registry must return None without panic"
    );
}

/// EC-002: slug_for called with never-registered OrgId returns None; no log.
///
/// BC-3.1.001 edge case 2.
#[test]
fn test_BC_3_1_001_ec002_slug_for_unregistered_id_returns_none() {
    let registry = OrgRegistry::new();
    let phantom = org_id();
    let result = registry.slug_for(&phantom);
    assert_eq!(
        result, None,
        "EC-002: slug_for of never-registered OrgId must return None"
    );
}

/// EC-004: Slug at maximum length (64 chars of `a-zA-Z0-9_-`) registers and resolves correctly.
///
/// BC-3.1.001 edge case 4.
#[test]
fn test_BC_3_1_001_ec004_max_length_slug_registers_and_resolves() {
    let max_slug_str: String = "a".repeat(64);
    let s = slug(&max_slug_str);
    let id = org_id();

    let registry = OrgRegistry::new();
    registry
        .register(s.clone(), id)
        .expect("max-length slug registration must succeed");

    assert_eq!(
        registry.resolve(&s),
        Some(id),
        "EC-004: max-length slug must resolve after registration"
    );
    assert_eq!(
        registry.slug_for(&id),
        Some(s),
        "EC-004: id registered with max-length slug must resolve back"
    );
}

/// BC-3.1.001 invariant 2: BiMap equivalence — resolve(slug)==Some(id) iff slug_for(id)==Some(slug).
///
/// Tests the double-implication for multiple entries.
#[test]
fn test_BC_3_1_001_invariant_bimap_equivalence() {
    let registry = OrgRegistry::new();
    let pairs: Vec<(OrgSlug, OrgId)> = vec![
        (slug("alpha-org"), org_id()),
        (slug("beta-org"), org_id()),
        (slug("gamma-org"), org_id()),
    ];

    for (s, id) in &pairs {
        registry
            .register(s.clone(), *id)
            .expect("registration must succeed");
    }

    for (s, id) in &pairs {
        // Forward direction: resolve(slug) == Some(id)
        assert_eq!(
            registry.resolve(s),
            Some(*id),
            "invariant 2: resolve({}) must return Some(id)",
            s
        );
        // Reverse direction: slug_for(id) == Some(slug)
        assert_eq!(
            registry.slug_for(id),
            Some(s.clone()),
            "invariant 2: slug_for(id) must return Some({})",
            s
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// BC-3.1.003 — OrgRegistry Maintains Strict Bijectivity at All Times
// ═══════════════════════════════════════════════════════════════════════════════

/// TV-3.1.003-01: After register(slug-A, uuid-A), both resolve and slug_for work.
///
/// BC-3.1.003 postcondition 1 / basic bijectivity.
#[test]
fn test_BC_3_1_003_tv_01_basic_bijectivity() {
    let registry = OrgRegistry::new();
    let uuid_a = org_id_from_v7_bytes(UUID_A_BYTES);
    let s = slug("slug-a");

    registry
        .register(s.clone(), uuid_a)
        .expect("registration must succeed");

    assert_eq!(
        registry.resolve(&s),
        Some(uuid_a),
        "TV-3.1.003-01: resolve(slug-A) must return Some(uuid-A)"
    );
    assert_eq!(
        registry.slug_for(&uuid_a),
        Some(s),
        "TV-3.1.003-01: slug_for(uuid-A) must return Some(slug-A)"
    );
}

/// TV-3.1.003-02: Second register with same slug / different uuid returns Err(SlugConflict).
///
/// BC-3.1.003 postcondition 1 / EC-001 / TV-3.1.003-02.
#[test]
fn test_BC_3_1_003_tv_02_slug_conflict_rejected() {
    let registry = OrgRegistry::new();
    let uuid_a = org_id_from_v7_bytes(UUID_A_BYTES);
    let uuid_b = org_id_from_v7_bytes(UUID_B_BYTES);
    let s = slug("acme-corp");

    registry
        .register(s.clone(), uuid_a)
        .expect("first registration must succeed");
    let result = registry.register(s.clone(), uuid_b);

    assert!(
        matches!(result, Err(RegistrationError::SlugConflict { .. })),
        "TV-3.1.003-02: second register with same slug/different id must return Err(SlugConflict), got {:?}",
        result
    );
    // Registry must be unchanged — slug still maps to uuid_a
    assert_eq!(
        registry.resolve(&s),
        Some(uuid_a),
        "TV-3.1.003-02: slug must still map to original uuid-A after rejected registration"
    );
}

/// TV-3.1.003-03: Second register with same uuid / different slug returns Err(IdConflict).
///
/// BC-3.1.003 EC-002 / TV-3.1.003-03.
#[test]
fn test_BC_3_1_003_tv_03_id_conflict_rejected() {
    let registry = OrgRegistry::new();
    let uuid_a = org_id_from_v7_bytes(UUID_A_BYTES);
    let s_a = slug("slug-a");
    let s_b = slug("slug-b");

    registry
        .register(s_a.clone(), uuid_a)
        .expect("first registration must succeed");
    let result = registry.register(s_b.clone(), uuid_a);

    assert!(
        matches!(result, Err(RegistrationError::IdConflict { .. })),
        "TV-3.1.003-03: second register with different slug/same id must return Err(IdConflict), got {:?}",
        result
    );
    // Registry must be unchanged — uuid_a still maps to slug-a
    assert_eq!(
        registry.slug_for(&uuid_a),
        Some(s_a),
        "TV-3.1.003-03: uuid-A must still map to slug-A after rejected registration"
    );
}

/// BC-3.1.003 EC-001: SlugConflict leaves registry unchanged.
///
/// After rejected slug-collision, resolve(slug-A) == Some(uuid-A) and slug-B was never inserted.
#[test]
fn test_BC_3_1_003_ec001_slug_conflict_preserves_registry() {
    let registry = OrgRegistry::new();
    let uuid_a = org_id_from_v7_bytes(UUID_A_BYTES);
    let uuid_b = org_id_from_v7_bytes(UUID_B_BYTES);
    let s_a = slug("slug-a");

    registry
        .register(s_a.clone(), uuid_a)
        .expect("first registration must succeed");
    let size_before = registry.len();

    let _ = registry.register(s_a.clone(), uuid_b); // must fail (SlugConflict)

    assert_eq!(
        registry.len(),
        size_before,
        "EC-001: size must not change after SlugConflict"
    );
    assert_eq!(
        registry.resolve(&s_a),
        Some(uuid_a),
        "EC-001: slug-A must still map to uuid-A"
    );
    // uuid-B was rejected; it must not appear in reverse map
    assert_eq!(
        registry.slug_for(&uuid_b),
        None,
        "EC-001: uuid-B was rejected; slug_for must return None"
    );
}

/// BC-3.1.003 EC-002: IdConflict leaves registry unchanged.
///
/// After rejected uuid-collision, slug_for(uuid-A) == Some(slug-A) and slug-B was never inserted.
#[test]
fn test_BC_3_1_003_ec002_id_conflict_preserves_registry() {
    let registry = OrgRegistry::new();
    let uuid_a = org_id_from_v7_bytes(UUID_A_BYTES);
    let s_a = slug("slug-a");
    let s_b = slug("slug-b");

    registry
        .register(s_a.clone(), uuid_a)
        .expect("first registration must succeed");
    let size_before = registry.len();

    let _ = registry.register(s_b.clone(), uuid_a); // must fail (IdConflict)

    assert_eq!(
        registry.len(),
        size_before,
        "EC-002: size must not change after IdConflict"
    );
    assert_eq!(
        registry.slug_for(&uuid_a),
        Some(s_a),
        "EC-002: uuid-A must still map to slug-A"
    );
    assert_eq!(
        registry.resolve(&s_b),
        None,
        "EC-002: slug-B was rejected; resolve must return None"
    );
}

/// BC-3.1.003 EC-004: Single-entry registry satisfies bijectivity trivially.
#[test]
fn test_BC_3_1_003_ec004_single_entry_bijection_holds() {
    let registry = OrgRegistry::new();
    let id = org_id();
    let s = slug("only-org");

    registry
        .register(s.clone(), id)
        .expect("registration must succeed");

    assert_eq!(
        registry.len(),
        1,
        "EC-004: registry must contain exactly 1 entry"
    );
    assert_eq!(registry.resolve(&s), Some(id));
    assert_eq!(registry.slug_for(&id), Some(s));
}

/// BC-3.1.003 EC-005: Registry with 100 entries — forward and reverse lengths are equal.
///
/// BC-3.1.003 postcondition 3: "forward map entry count always equals reverse map entry count."
#[test]
fn test_BC_3_1_003_ec005_100_entries_forward_eq_reverse_len() {
    let registry = OrgRegistry::new();
    for i in 0..100u32 {
        let s = OrgSlug::new(format!("org-{:03}", i)).unwrap();
        let id = org_id();
        registry.register(s, id).expect("registration must succeed");
    }
    // len() reports forward map length; bijectivity guarantees reverse is equal.
    assert_eq!(
        registry.len(),
        100,
        "EC-005: registry must contain exactly 100 entries"
    );
}

/// BC-3.1.003 invariant 3: forward-map entry count always equals reverse-map entry count.
///
/// Verified by checking len() after each of N sequential registrations.
#[test]
fn test_BC_3_1_003_invariant_forward_len_eq_reverse_len() {
    let registry = OrgRegistry::new();
    for i in 0..20u32 {
        let s = OrgSlug::new(format!("org-{}", i)).unwrap();
        let id = org_id();
        registry.register(s, id).expect("registration must succeed");
        // After each registration, len must equal i+1.
        assert_eq!(
            registry.len(),
            (i + 1) as usize,
            "invariant 3: len must equal {} after {} registrations",
            i + 1,
            i + 1
        );
    }
}

/// VP-3.1.003-01 / AC-5: proptest bijection size invariant.
///
/// After N valid registrations, forward_len == reverse_len (== N).
/// Uses 1000 cases as required by the BC proptest requirement.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    /// AC-5 (traces to BC-3.1.003 invariant 1 + postcondition 3):
    /// A sequence of N unique registrations leaves the registry in a state where
    /// `len()` equals N. Since BiMap enforces bijection, forward and reverse counts
    /// are always equal.
    #[test]
    fn test_BC_3_1_003_proptest_bijection_size_invariant(n in 1usize..=30) {
        let registry = OrgRegistry::new();
        for i in 0..n {
            // Generate unique slugs by index — avoids collision across iterations.
            let s = OrgSlug::new(format!("org-prop-{}", i)).unwrap();
            let id = org_id();
            registry.register(s, id).expect("registration must succeed");
        }
        prop_assert_eq!(
            registry.len(),
            n,
            "VP-3.1.003-01: len() must equal number of successful registrations"
        );
        prop_assert!(
            !registry.is_empty(),
            "VP-3.1.003-01: registry must not be empty after registrations"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// BC-3.1.004 — OrgRegistry Rejects Duplicate Slugs and UUIDs at Registration
// ═══════════════════════════════════════════════════════════════════════════════

/// TV-3.1.004-01: SlugConflict error contains slug, existing_id, and attempted_id.
///
/// BC-3.1.004 postcondition 2 / VP-3.1.004-02 / AC-6.
#[test]
fn test_BC_3_1_004_tv_01_slug_conflict_error_fields() {
    let registry = OrgRegistry::new();
    let uuid_a = org_id_from_v7_bytes(UUID_A_BYTES);
    let uuid_b = org_id_from_v7_bytes(UUID_B_BYTES);
    let s = slug("acme-corp");

    registry
        .register(s.clone(), uuid_a)
        .expect("first registration must succeed");
    let result = registry.register(s.clone(), uuid_b);

    match result {
        Err(RegistrationError::SlugConflict {
            slug: conflict_slug,
            existing_id,
            attempted_id,
        }) => {
            assert_eq!(
                conflict_slug, s,
                "TV-3.1.004-01: SlugConflict must identify the conflicting slug"
            );
            assert_eq!(
                existing_id, uuid_a,
                "TV-3.1.004-01: SlugConflict must identify the existing OrgId"
            );
            assert_eq!(
                attempted_id, uuid_b,
                "TV-3.1.004-01: SlugConflict must identify the attempted OrgId"
            );
        }
        other => panic!("TV-3.1.004-01: expected Err(SlugConflict), got {:?}", other),
    }
}

/// TV-3.1.004-02: IdConflict error contains id, existing_slug, and attempted_slug.
///
/// BC-3.1.004 postcondition 3 / VP-3.1.004-03 / AC-7.
#[test]
fn test_BC_3_1_004_tv_02_id_conflict_error_fields() {
    let registry = OrgRegistry::new();
    let uuid_a = org_id_from_v7_bytes(UUID_A_BYTES);
    let s_a = slug("acme-corp");
    let s_b = slug("beta-inc");

    registry
        .register(s_a.clone(), uuid_a)
        .expect("first registration must succeed");
    let result = registry.register(s_b.clone(), uuid_a);

    match result {
        Err(RegistrationError::IdConflict {
            id,
            existing_slug,
            attempted_slug,
        }) => {
            assert_eq!(
                id, uuid_a,
                "TV-3.1.004-02: IdConflict must identify the conflicting OrgId"
            );
            assert_eq!(
                existing_slug, s_a,
                "TV-3.1.004-02: IdConflict must identify the existing slug"
            );
            assert_eq!(
                attempted_slug, s_b,
                "TV-3.1.004-02: IdConflict must identify the attempted slug"
            );
        }
        other => panic!("TV-3.1.004-02: expected Err(IdConflict), got {:?}", other),
    }
}

/// TV-3.1.004-03: After rejected registration, resolve returns pre-rejection state.
///
/// BC-3.1.004 postcondition 2: "pre-call state is fully preserved."
/// VP-3.1.004-04 partial.
#[test]
fn test_BC_3_1_004_tv_03_no_partial_state_after_rejection() {
    let registry = OrgRegistry::new();
    let uuid_a = org_id_from_v7_bytes(UUID_A_BYTES);
    let uuid_b = org_id_from_v7_bytes(UUID_B_BYTES);
    let s = slug("acme-corp");

    registry
        .register(s.clone(), uuid_a)
        .expect("first registration must succeed");
    // Rejected registration — slug already bound to uuid_a
    let _ = registry.register(s.clone(), uuid_b);

    // Pre-call state must be preserved
    assert_eq!(
        registry.resolve(&s),
        Some(uuid_a),
        "TV-3.1.004-03: resolve(slug) must return uuid-A after rejected registration"
    );
    assert_eq!(
        registry.slug_for(&uuid_a),
        Some(s),
        "TV-3.1.004-03: slug_for(uuid-A) must still return original slug"
    );
    // uuid_b must not appear
    assert_eq!(
        registry.slug_for(&uuid_b),
        None,
        "TV-3.1.004-03: uuid-B from rejected registration must not appear in registry"
    );
}

/// AC-8 / BC-3.1.004 EC-003: Exact duplicate re-registration returns Ok (idempotent per D-050).
///
/// BC-3.1.004 postcondition 4: "register(slug, id) with the exact same (slug, id) pair
/// already registered returns Ok(())."
#[test]
fn test_BC_3_1_004_ac8_exact_duplicate_is_idempotent() {
    let registry = OrgRegistry::new();
    let id = org_id_from_v7_bytes(UUID_A_BYTES);
    let s = slug("acme-corp");

    registry
        .register(s.clone(), id)
        .expect("first registration must succeed");
    let result = registry.register(s.clone(), id);

    assert!(
        result.is_ok(),
        "AC-8/EC-003: exact duplicate (slug, id) registration must return Ok (idempotent per D-050), got {:?}",
        result
    );
    // Registry size must remain 1 (not incremented)
    assert_eq!(
        registry.len(),
        1,
        "AC-8: idempotent re-registration must not change registry size"
    );
}

/// BC-3.1.004 EC-001: register(slug-A, uuid-A); register(slug-A, uuid-B) returns Err(SlugConflict).
///
/// The variant discriminant must be SlugConflict, not IdConflict.
#[test]
fn test_BC_3_1_004_ec001_slug_conflict_is_err_slug_conflict() {
    let registry = OrgRegistry::new();
    let uuid_a = org_id_from_v7_bytes(UUID_A_BYTES);
    let uuid_b = org_id_from_v7_bytes(UUID_B_BYTES);
    let s = slug("slug-a");

    registry
        .register(s.clone(), uuid_a)
        .expect("first registration must succeed");
    let result = registry.register(s.clone(), uuid_b);

    assert!(
        matches!(result, Err(RegistrationError::SlugConflict { .. })),
        "EC-001: same slug / different id must return Err(SlugConflict), got {:?}",
        result
    );
}

/// BC-3.1.004 EC-002: register(slug-A, uuid-A); register(slug-B, uuid-A) returns Err(IdConflict).
///
/// The variant discriminant must be IdConflict, not SlugConflict.
#[test]
fn test_BC_3_1_004_ec002_id_conflict_is_err_id_conflict() {
    let registry = OrgRegistry::new();
    let uuid_a = org_id_from_v7_bytes(UUID_A_BYTES);
    let s_a = slug("slug-a");
    let s_b = slug("slug-b");

    registry
        .register(s_a.clone(), uuid_a)
        .expect("first registration must succeed");
    let result = registry.register(s_b, uuid_a);

    assert!(
        matches!(result, Err(RegistrationError::IdConflict { .. })),
        "EC-002: different slug / same id must return Err(IdConflict), got {:?}",
        result
    );
}

/// BC-3.1.004 EC-005: A valid pair registers successfully after a rejected attempt.
///
/// Rejected attempt leaves no trace; subsequent valid registration proceeds normally.
#[test]
fn test_BC_3_1_004_ec005_valid_after_rejected_attempt() {
    let registry = OrgRegistry::new();
    let uuid_a = org_id_from_v7_bytes(UUID_A_BYTES);
    let uuid_b = org_id_from_v7_bytes(UUID_B_BYTES);
    let s_a = slug("org-a");
    let s_b = slug("org-b");

    // Register first pair
    registry
        .register(s_a.clone(), uuid_a)
        .expect("first registration must succeed");
    // Attempt a conflicting registration (slug-a + uuid-b → SlugConflict)
    let _ = registry.register(s_a.clone(), uuid_b);
    // Now register a genuinely new pair — must succeed
    registry
        .register(s_b.clone(), uuid_b)
        .expect("EC-005: valid pair must register after a rejected attempt");

    assert_eq!(
        registry.resolve(&s_b),
        Some(uuid_b),
        "EC-005: new pair must resolve correctly"
    );
    assert_eq!(
        registry.slug_for(&uuid_b),
        Some(s_b),
        "EC-005: new pair must reverse-resolve"
    );
    // Original pair untouched
    assert_eq!(
        registry.resolve(&s_a),
        Some(uuid_a),
        "EC-005: original pair must be unchanged"
    );
}

/// BC-3.1.004 invariant 1: register is atomic — either fully applied or entirely unchanged.
///
/// Verifies that after a SlugConflict rejection, neither the attempted id nor the
/// partial slug appears anywhere in the registry.
#[test]
fn test_BC_3_1_004_invariant1_register_atomic_no_partial_state() {
    let registry = OrgRegistry::new();
    let uuid_a = org_id_from_v7_bytes(UUID_A_BYTES);
    let uuid_b = org_id_from_v7_bytes(UUID_B_BYTES);
    let s = slug("acme");

    registry
        .register(s.clone(), uuid_a)
        .expect("initial registration must succeed");

    // Attempt to re-bind "acme" to a different uuid (SlugConflict)
    let _ = registry.register(s.clone(), uuid_b);

    // Registry must be fully in pre-call state
    assert_eq!(
        registry.len(),
        1,
        "invariant 1: size unchanged after rejected register"
    );
    assert_eq!(
        registry.resolve(&s),
        Some(uuid_a),
        "invariant 1: slug still maps to original id"
    );
    assert_eq!(
        registry.slug_for(&uuid_b),
        None,
        "invariant 1: attempted id must not appear in reverse map"
    );
}

/// BC-3.1.004 invariant 2: no silent last-write-wins.
///
/// Re-registering an existing slug with a new UUID is ALWAYS an error.
/// This test uses multiple conflicting slugs to assert the invariant across several attempts.
#[test]
fn test_BC_3_1_004_invariant2_no_silent_overwrite() {
    let registry = OrgRegistry::new();
    let uuid_a = org_id_from_v7_bytes(UUID_A_BYTES);
    let s = slug("contested-slug");

    registry
        .register(s.clone(), uuid_a)
        .expect("initial registration must succeed");

    // Attempt five different uuid_b values — each must return Err, never Ok
    for _ in 0..5 {
        let new_id = org_id();
        let result = registry.register(s.clone(), new_id);
        assert!(
            matches!(result, Err(RegistrationError::SlugConflict { .. })),
            "invariant 2: re-registering existing slug with any new id must Err(SlugConflict)"
        );
    }
    // Original mapping must be intact throughout
    assert_eq!(
        registry.resolve(&s),
        Some(uuid_a),
        "invariant 2: slug must still map to original id after all rejected attempts"
    );
}

/// BC-3.1.004 invariant 3: SlugConflict Display contains both slug and both UUIDs.
///
/// VP-3.1.004-02: error message identifies existing UUID and attempted UUID.
#[test]
fn test_BC_3_1_004_invariant3_slug_conflict_error_display() {
    let registry = OrgRegistry::new();
    let uuid_a = org_id_from_v7_bytes(UUID_A_BYTES);
    let uuid_b = org_id_from_v7_bytes(UUID_B_BYTES);
    let s = slug("acme-corp");

    registry
        .register(s.clone(), uuid_a)
        .expect("registration must succeed");
    let err = registry
        .register(s.clone(), uuid_b)
        .expect_err("second registration must fail");

    let display = err.to_string();
    assert!(
        display.contains("acme-corp"),
        "invariant 3: SlugConflict Display must contain slug 'acme-corp'; got: '{}'",
        display
    );
    // Both UUIDs must appear in the error message
    let uuid_a_str = uuid_a.to_string();
    let uuid_b_str = uuid_b.to_string();
    assert!(
        display.contains(&uuid_a_str),
        "invariant 3: SlugConflict Display must contain existing_id '{}'; got: '{}'",
        uuid_a_str,
        display
    );
    assert!(
        display.contains(&uuid_b_str),
        "invariant 3: SlugConflict Display must contain attempted_id '{}'; got: '{}'",
        uuid_b_str,
        display
    );
}

/// BC-3.1.004 invariant 3: IdConflict Display contains both slugs and UUID.
///
/// VP-3.1.004-03: error message identifies existing slug and attempted slug.
#[test]
fn test_BC_3_1_004_invariant3_id_conflict_error_display() {
    let registry = OrgRegistry::new();
    let uuid_a = org_id_from_v7_bytes(UUID_A_BYTES);
    let s_a = slug("acme-corp");
    let s_b = slug("beta-inc");

    registry
        .register(s_a.clone(), uuid_a)
        .expect("registration must succeed");
    let err = registry
        .register(s_b.clone(), uuid_a)
        .expect_err("second registration must fail");

    let display = err.to_string();
    assert!(
        display.contains("acme-corp"),
        "invariant 3: IdConflict Display must contain existing slug 'acme-corp'; got: '{}'",
        display
    );
    assert!(
        display.contains("beta-inc"),
        "invariant 3: IdConflict Display must contain attempted slug 'beta-inc'; got: '{}'",
        display
    );
    let uuid_a_str = uuid_a.to_string();
    assert!(
        display.contains(&uuid_a_str),
        "invariant 3: IdConflict Display must contain conflicting id '{}'; got: '{}'",
        uuid_a_str,
        display
    );
}

/// VP-3.1.004-01: Registry size is unchanged after any Err return from register.
///
/// Proptest: apply a random sequence of valid registrations followed by a conflicting
/// one; assert size is unchanged by the conflict.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn test_BC_3_1_004_proptest_size_unchanged_on_error(n in 1usize..=20) {
        let registry = OrgRegistry::new();
        let mut last_slug: Option<OrgSlug> = None;
        let mut last_id: Option<OrgId> = None;

        // Populate n valid entries
        for i in 0..n {
            let s = OrgSlug::new(format!("prop-org-{}", i)).unwrap();
            let id = org_id();
            registry.register(s.clone(), id).expect("registration must succeed");
            last_slug = Some(s);
            last_id = Some(id);
        }

        let size_before = registry.len();

        // Attempt a SlugConflict (same slug, different id)
        if let (Some(s), Some(existing_id)) = (last_slug, last_id) {
            let conflicting_id = org_id();
            prop_assume!(conflicting_id != existing_id);
            let _ = registry.register(s, conflicting_id);
            prop_assert_eq!(
                registry.len(),
                size_before,
                "VP-3.1.004-01: size must be unchanged after Err from register"
            );
        }
    }

    /// VP-3.1.004-04: After N successful registrations and one rejected registration,
    /// resolve produces correct results for all N successful pairs.
    #[test]
    fn test_BC_3_1_004_proptest_successful_resolve_after_rejection(n in 2usize..=15) {
        let registry = OrgRegistry::new();

        // Register n unique pairs and remember them
        let pairs: Vec<(OrgSlug, OrgId)> = (0..n)
            .map(|i| {
                let s = OrgSlug::new(format!("prop-r-org-{}", i)).unwrap();
                let id = org_id();
                registry.register(s.clone(), id).expect("registration must succeed");
                (s, id)
            })
            .collect();

        // Inject a failing registration (re-use slug[0] with a new id)
        let (conflict_slug, original_id) = pairs[0].clone();
        let new_id = org_id();
        prop_assume!(new_id != original_id);
        let _ = registry.register(conflict_slug, new_id);

        // All n successful pairs must still resolve correctly
        for (s, id) in &pairs {
            prop_assert_eq!(
                registry.resolve(s),
                Some(*id),
                "VP-3.1.004-04: resolve({}) must return correct id after rejected registration",
                s
            );
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Ancillary / helper-method tests
// ═══════════════════════════════════════════════════════════════════════════════

/// OrgRegistry::new() creates an empty registry.
#[test]
fn test_BC_3_1_004_is_empty_new_registry() {
    let registry = OrgRegistry::new();
    assert!(registry.is_empty(), "new() must produce an empty registry");
    assert_eq!(registry.len(), 0, "new() registry len must be 0");
}

/// len() increments with each successful registration.
#[test]
fn test_BC_3_1_004_len_reflects_registrations() {
    let registry = OrgRegistry::new();
    for i in 0..5u32 {
        let s = OrgSlug::new(format!("len-org-{}", i)).unwrap();
        let id = org_id();
        registry.register(s, id).expect("registration must succeed");
        assert_eq!(
            registry.len(),
            (i + 1) as usize,
            "len() must be {} after {} registrations",
            i + 1,
            i + 1
        );
    }
    assert!(
        !registry.is_empty(),
        "non-empty registry must not report is_empty()"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// BC-3.1.001 EC-005 — Concurrent reads are safe (BC-3.1.004 concurrency)
// ═══════════════════════════════════════════════════════════════════════════════

/// EC-005: Concurrent reads from multiple threads return consistent results.
///
/// OrgRegistry is Arc<RwLock<BiMap<...>>>. This test populates the registry
/// on the main thread, then spawns N reader threads that all call resolve() and
/// slug_for() concurrently. All results must be consistent with the registered state.
///
/// BC-3.1.001 EC-005 / BC-3.1.003 (concurrency safe by construction).
#[test]
fn test_BC_3_1_001_ec005_concurrent_reads_are_safe() {
    let registry = Arc::new(OrgRegistry::new());

    // Populate registry on main thread
    let n: usize = 20;
    let mut pairs: Vec<(OrgSlug, OrgId)> = Vec::with_capacity(n);
    for i in 0..n {
        let s = OrgSlug::new(format!("concurrent-org-{:02}", i)).unwrap();
        let id = org_id();
        registry
            .register(s.clone(), id)
            .expect("registration must succeed");
        pairs.push((s, id));
    }

    let pairs = Arc::new(pairs);

    // Spawn 8 concurrent reader threads
    let handles: Vec<_> = (0..8)
        .map(|_| {
            let reg = Arc::clone(&registry);
            let ps = Arc::clone(&pairs);
            thread::spawn(move || {
                for (s, id) in ps.iter() {
                    let resolved = reg.resolve(s);
                    assert_eq!(
                        resolved,
                        Some(*id),
                        "concurrent read: resolve must return correct id"
                    );
                    let slug_result = reg.slug_for(id);
                    assert_eq!(
                        slug_result,
                        Some(s.clone()),
                        "concurrent read: slug_for must return correct slug"
                    );
                }
            })
        })
        .collect();

    for h in handles {
        h.join().expect("reader thread must not panic");
    }
}
