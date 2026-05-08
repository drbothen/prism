//! Tests for S-3.1.01 — OrgId UUID v7 newtype.
//!
//! Traces to: BC-3.1.001 (OrgRegistry Bijective Slug/UUID Resolution)
//! Verification properties covered: VP-063, VP-064, VP-065
//!
//! `OrgId::from_uuid_v7` is implemented and panics on non-v7 input (see
//! `crates/prism-core/src/ids.rs:83`). `OrgId` also implements `Display`,
//! producing the hyphenated lowercase UUID string (see `ids.rs:94`).
//!
//! # Test inventory
//!
//! | Test function                                      | AC  | Status  |
//! |----------------------------------------------------|-----|---------|
//! | test_bc_3_1_001_ac_1_new_generates_v7_uuid         | AC-1| passes  |
//! | test_bc_3_1_001_ac_1_from_uuid_panics_on_v4        | AC-1| passes  |
//! | test_bc_3_1_001_ac_2_re_export_compiles            | AC-2| passes  |
//! | test_bc_3_1_001_ac_3_hashmap_key_compiles          | AC-3| passes  |
//! | test_bc_3_1_001_ac_3_derives_equality              | AC-3| passes  |
//! | test_bc_3_1_001_ac_3_derives_clone_copy            | AC-3| passes  |
//! | test_bc_3_1_001_ac_3_serde_round_trip_json         | AC-3| passes  |
//! | test_bc_3_1_001_ac_4_display_hyphenated_lowercase  | AC-4| passes  |
//! | test_bc_3_1_001_ec_001_from_uuid_v4_panics         | EC-001| passes |
//! | test_bc_3_1_001_ec_002_two_new_both_valid_v7       | EC-002| passes |
//! | test_bc_3_1_001_ec_003_hashmap_key_stores_values   | EC-003| passes |

use prism_core::OrgId;
use std::collections::HashMap;
use uuid::Uuid;

// ── AC-1: OrgId::new() generates a v7 UUID ─────────────────────────────────
//
// BC-3.1.001 precondition 3: "OrgId values are UUID v7; UUID v4 is prohibited."
// VP-063: OrgId::new() always produces a UUID whose version field is 7.

/// AC-1 (GREEN-BY-DESIGN at Red Gate): `OrgId::new()` returns a value wrapping a v7 UUID.
///
/// `uuid_v7_newtype!` already calls `Uuid::now_v7()` in `new()`. This test will pass
/// at Red Gate because the stub correctly implements `new()`. It documents the required
/// invariant and acts as a regression guard.
#[test]
fn test_bc_3_1_001_ac_1_new_generates_v7_uuid() {
    let org_id = OrgId::new();
    let inner = org_id.as_uuid();
    assert_eq!(
        inner.get_version_num(),
        7,
        "OrgId::new() must produce a UUID v7 (version = 7), got version {}",
        inner.get_version_num()
    );
}

/// AC-1: `from_uuid_v7(v4_uuid)` must panic — v4 is prohibited.
///
/// BC-3.1.001 precondition 3 requires that constructing an OrgId from a non-v7 UUID
/// panics. `OrgId::from_uuid_v7` enforces this: it panics with "not a UUID v7" when
/// given a v4 UUID. The test calls `from_uuid_v7` directly (not the unchecked
/// `from_uuid`) and relies on that enforcement.
#[test]
#[should_panic(expected = "not a UUID v7")]
fn test_bc_3_1_001_ac_1_from_uuid_panics_on_v4() {
    // Construct a deterministic v4 UUID for reproducible failure output.
    let v4_uuid =
        Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").expect("parse known v4 UUID");
    assert_eq!(
        v4_uuid.get_version_num(),
        4,
        "test precondition: UUID must be v4"
    );
    // This must panic with a message containing "not a UUID v7".
    let _org_id = OrgId::from_uuid_v7(v4_uuid);
}

// ── AC-2: OrgId is re-exported from prism_core ─────────────────────────────
//
// BC-3.1.001 precondition 3 / story spec: `use prism_core::OrgId` must compile.
// VP-064: re-export stability.

/// AC-2 (GREEN-BY-DESIGN at Red Gate): `use prism_core::OrgId` compiles.
///
/// The stub already re-exports OrgId from lib.rs. This test confirms the re-export is
/// present and usable. It will pass at Red Gate. It acts as a regression guard against
/// accidental removal of the re-export.
#[test]
fn test_bc_3_1_001_ac_2_re_export_compiles() {
    // If OrgId is not re-exported, this file will not compile.
    // The fact that this test runs proves AC-2 is satisfied.
    let _id: OrgId = OrgId::new();
    // Additional: confirm we can call methods introduced by the macro.
    let uuid = _id.as_uuid();
    assert_eq!(
        uuid.get_version_num(),
        7,
        "re-exported OrgId must produce v7"
    );
}

// ── AC-3: Required derives are present ─────────────────────────────────────
//
// BC-3.1.001 invariant 1: OrgId must derive Debug, Clone, Copy, PartialEq, Eq,
// Hash, Serialize, Deserialize.
// EC-003: OrgId can be stored as a HashMap key (requires Hash + Eq).
// VP-065: trait-derive completeness.

/// AC-3 (GREEN-BY-DESIGN): `OrgId` can be stored as a `HashMap` key.
///
/// Requires `Hash + Eq`, both of which are derived by `uuid_v7_newtype!`. Compilation
/// of this test is proof that the derives are present. Will pass at Red Gate.
#[test]
fn test_bc_3_1_001_ac_3_hashmap_key_compiles() {
    let mut map: HashMap<OrgId, String> = HashMap::new();
    let id = OrgId::new();
    map.insert(id, "test-org".to_string());
    assert_eq!(map.get(&id).map(String::as_str), Some("test-org"));
}

/// AC-3 (GREEN-BY-DESIGN): `PartialEq + Eq` are derived correctly.
///
/// Two OrgIds constructed from the same underlying Uuid must compare equal.
#[test]
fn test_bc_3_1_001_ac_3_derives_equality() {
    let uuid = Uuid::now_v7();
    let a = OrgId::from_uuid(uuid);
    let b = OrgId::from_uuid(uuid);
    assert_eq!(a, b, "same underlying Uuid must produce equal OrgIds");
    let c = OrgId::new();
    assert_ne!(a, c, "distinct OrgIds must not compare equal");
}

/// AC-3 (GREEN-BY-DESIGN): `Clone + Copy` semantics are correct.
///
/// Copy means moving `a` into `_b` still leaves `a` accessible.
#[test]
#[allow(clippy::clone_on_copy)] // intentional: test documents that Clone is derived (AC-3)
fn test_bc_3_1_001_ac_3_derives_clone_copy() {
    let a = OrgId::new();
    let b = a; // Copy, not move
    let c = a.clone(); // Clone
    assert_eq!(a, b);
    assert_eq!(a, c);
}

/// AC-3 (GREEN-BY-DESIGN): `Serialize + Deserialize` round-trip via JSON.
///
/// The macro derives serde traits. A serialized OrgId must deserialize back to the
/// same value.
#[test]
fn test_bc_3_1_001_ac_3_serde_round_trip_json() {
    let original = OrgId::new();
    let json = serde_json::to_string(&original).expect("OrgId must serialize to JSON");
    let recovered: OrgId = serde_json::from_str(&json).expect("OrgId must deserialize from JSON");
    assert_eq!(
        original, recovered,
        "serde round-trip must preserve the OrgId value"
    );
}

// ── AC-4: Display produces hyphenated lowercase UUID string ─────────────────
//
// BC-3.1.001 invariant 3 / story spec: `OrgId::from_uuid_v7(known_uuid).to_string()`
// must equal the hyphenated lowercase UUID string.

/// AC-4: `OrgId` Display must produce hyphenated lowercase UUID.
///
/// `OrgId` implements `std::fmt::Display` (see `ids.rs:94`), producing the bare
/// hyphenated lowercase UUID string.
#[test]
fn test_bc_3_1_001_ac_4_display_hyphenated_lowercase() {
    // Build a known v7 UUID from fixed bytes for deterministic output.
    // Version nibble is the top 4 bits of byte 6 (must be 0x70 | ... for v7).
    // Using Uuid::now_v7() since we can't easily hand-craft a v7 from raw bytes
    // without knowing the exact bit layout; the Display contract is format-independent.
    let uuid = Uuid::now_v7();
    let org_id = OrgId::from_uuid(uuid);
    let expected = uuid.to_string(); // hyphenated lowercase, e.g. "018e3f71-5c6d-7..."

    // GREEN GATE: uses Display (impl added in S-3.1.01 Green Gate).
    // Display must produce the bare hyphenated lowercase UUID string.
    let display_output = format!("{}", org_id);
    assert_eq!(
        display_output, expected,
        "OrgId Display output '{}' does not match the required hyphenated \
         lowercase UUID string '{}'",
        display_output, expected
    );
}

// ── EC-001: from_uuid(v4) panics ─────────────────────────────────────────────
//
// Story spec EC-001: "OrgId::from_uuid_v7(uuid_v4) called with a v4 UUID →
// Panics with a clear message identifying the version mismatch."

/// EC-001: constructing `OrgId` from a v4 UUID via `from_uuid_v7` must panic.
///
/// `OrgId::from_uuid_v7` enforces the UUID version and panics with "not a UUID v7"
/// when given a non-v7 UUID (see `ids.rs:83`).
#[test]
#[should_panic(expected = "not a UUID v7")]
fn test_bc_3_1_001_ec_001_from_uuid_v4_panics() {
    // A well-known v4 UUID (RFC 4122 nil-like with v4 version bits set).
    let v4_bytes: [u8; 16] = [
        0x55, 0x0e, 0x84, 0x00, // time_low
        0xe2, 0x9b, // time_mid
        0x41, 0xd4, // time_hi_and_version (version = 4 → top nibble = 0x4)
        0xa7, 0x16, // clock_seq
        0x44, 0x66, 0x55, 0x44, 0x00, 0x00, // node
    ];
    let v4_uuid = Uuid::from_bytes(v4_bytes);
    assert_eq!(
        v4_uuid.get_version_num(),
        4,
        "EC-001 test precondition: input UUID must be v4, got version {}",
        v4_uuid.get_version_num()
    );
    // MUST panic with message containing "not a UUID v7".
    let _org_id = OrgId::from_uuid_v7(v4_uuid);
}

// ── EC-002: Two OrgId::new() calls both produce valid v7 UUIDs ───────────────
//
// Story spec EC-002: "Both produce valid v7 UUIDs; monotonic ordering is best-effort."

/// EC-002 (GREEN-BY-DESIGN): Two rapid `OrgId::new()` calls both produce v7 UUIDs.
///
/// The story spec acknowledges monotonic ordering is best-effort for v7. This test
/// only asserts that both values are valid v7 UUIDs and that they are not equal (which
/// should hold in practice given the 80-bit random node component of UUIDv7).
#[test]
fn test_bc_3_1_001_ec_002_two_new_both_valid_v7() {
    let id1 = OrgId::new();
    let id2 = OrgId::new();
    assert_eq!(
        id1.as_uuid().get_version_num(),
        7,
        "first OrgId::new() must produce v7 UUID"
    );
    assert_eq!(
        id2.as_uuid().get_version_num(),
        7,
        "second OrgId::new() must produce v7 UUID"
    );
    // Not strictly required by the spec, but a valid v7 UUID should be unique
    // across separate calls due to the 80-bit random component.
    assert_ne!(
        id1, id2,
        "two OrgId::new() calls must produce distinct identifiers"
    );
}

// ── EC-003: OrgId can be stored as a HashMap key ─────────────────────────────
//
// Story spec EC-003: "OrgId stored as HashMap key — compiles and works; Hash + Eq derived."

/// EC-003 (GREEN-BY-DESIGN): OrgId works as a HashMap key with correct lookup semantics.
///
/// Inserts two entries and verifies that lookup by key returns the correct value.
/// This requires `Hash + Eq` to be correctly derived (they must be consistent:
/// `a == b` implies `hash(a) == hash(b)`).
#[test]
fn test_bc_3_1_001_ec_003_hashmap_key_stores_values() {
    let id_alpha = OrgId::new();
    let id_beta = OrgId::new();
    assert_ne!(
        id_alpha, id_beta,
        "test precondition: keys must be distinct"
    );

    let mut map: HashMap<OrgId, &str> = HashMap::new();
    map.insert(id_alpha, "alpha-org");
    map.insert(id_beta, "beta-org");

    assert_eq!(
        map.get(&id_alpha),
        Some(&"alpha-org"),
        "lookup by id_alpha must return 'alpha-org'"
    );
    assert_eq!(
        map.get(&id_beta),
        Some(&"beta-org"),
        "lookup by id_beta must return 'beta-org'"
    );
    assert_eq!(map.len(), 2, "map must contain exactly 2 entries");
}
