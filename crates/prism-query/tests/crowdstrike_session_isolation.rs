// BC-3.2.003 Invariant 1; VP-084; ADR-008 D-048; story S-3.2.08
//
// # Traceability
//
// - BC-3.2.003 precondition 4  — CrowdStrike session_registry NOT re-keyed; enforcement
//   at query-engine layer (D-048)
// - BC-3.2.003 invariant 1     — every session token entry carries an OrgId
// - BC-3.2.003 invariant 4     — structural keying required; probabilistic uniqueness
//   insufficient for formal isolation
// - BC-3.2.003 postcondition 2 — is_valid_session(org_b, token_a) returns invalid
// - VP-084                     — cross-org isolation property
// - ADR-008 §2.1 D-048         — UUID v7 org-temporal uniqueness rationale
// - S-3.2.08                   — story anchor
//
// # Red Gate
//
// All tests in this file are written BEFORE implementation.  Every test calls a stub
// function body of `todo!()` and therefore MUST FAIL with a panic.  Once the
// implementer phase is complete, all tests must turn GREEN with no changes to the test
// assertions themselves.

use prism_core::OrgId;
use prism_query::crowdstrike_session::{
    extract_org_id_from_session_id, generate_crowdstrike_session_id, xor_org_into_session_bytes,
};
use uuid::{Uuid, Version};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a deterministic OrgId from a fixed UUID v7 string so tests do not rely on
/// wall-clock calls and are therefore fully reproducible.
///
/// The UUID strings below are hand-crafted UUID v7 values (version nibble = 7,
/// variant bits = 10xxxxxx in byte 8).
fn org_a() -> OrgId {
    // 018e3f71-0001-7000-8000-000000000001 — version nibble 7, variant 10
    OrgId::from_uuid(
        Uuid::parse_str("018e3f71-0001-7000-8000-000000000001").expect("valid UUID literal"),
    )
}

fn org_b() -> OrgId {
    // 018e3f71-0002-7000-8000-000000000002 — same time prefix, different random bytes
    OrgId::from_uuid(
        Uuid::parse_str("018e3f71-0002-7000-8000-000000000002").expect("valid UUID literal"),
    )
}

// ---------------------------------------------------------------------------
// AC-001 — generate_crowdstrike_session_id embeds org_id (BC-3.2.003 inv 1, VP-084)
// ---------------------------------------------------------------------------

/// Generating a session ID for org_a and round-tripping through
/// extract_org_id_from_session_id must return Some(org_a).
///
/// Traces: BC-3.2.003 invariant 1, VP-084, AC-001, D-048
#[test]
fn test_BC_3_2_003_generate_embeds_org_a_roundtrip() {
    let id = generate_crowdstrike_session_id(org_a());
    let recovered = extract_org_id_from_session_id(&id);
    assert_eq!(
        recovered,
        Some(org_a()),
        "extract must recover org_a from a session ID generated for org_a"
    );
}

/// Generating a session ID for org_b and round-tripping through
/// extract_org_id_from_session_id must return Some(org_b).
///
/// Traces: BC-3.2.003 invariant 1, VP-084, AC-001, D-048
#[test]
fn test_BC_3_2_003_generate_embeds_org_b_roundtrip() {
    let id = generate_crowdstrike_session_id(org_b());
    let recovered = extract_org_id_from_session_id(&id);
    assert_eq!(
        recovered,
        Some(org_b()),
        "extract must recover org_b from a session ID generated for org_b"
    );
}

/// A session ID generated for org_a must NEVER be interpreted as belonging to org_b.
///
/// Traces: BC-3.2.003 postcondition 2, VP-084, AC-001, D-048
#[test]
fn test_BC_3_2_003_generate_org_a_never_returns_org_b() {
    let id = generate_crowdstrike_session_id(org_a());
    let recovered = extract_org_id_from_session_id(&id);
    assert_ne!(
        recovered,
        Some(org_b()),
        "extract on an org_a session ID must not return org_b"
    );
}

// ---------------------------------------------------------------------------
// AC-002 — xor_org_into_session_bytes is involutive (XOR round-trip)
// ---------------------------------------------------------------------------

/// XOR is its own inverse: xor(xor(base, org), org) == base.
///
/// Table-driven with a fixed UUID v7 base and three distinct OrgIds to cover
/// the property without a proptest dependency (proptest added below in AC-002-prop).
///
/// Traces: BC-3.2.003 invariant 4, AC-002
#[test]
fn test_BC_3_2_003_xor_org_involutive_table_driven() {
    let base = Uuid::parse_str("018e3f72-abcd-7ef0-8123-456789abcdef").expect("valid UUID literal");

    let orgs = [
        org_a(),
        org_b(),
        // all-zero OrgId (nil UUID) — EC-002: XOR with 0 is identity
        OrgId::from_uuid(Uuid::nil()),
    ];

    for org in orgs {
        let once = xor_org_into_session_bytes(base, org);
        let twice = xor_org_into_session_bytes(once, org);
        assert_eq!(
            twice, base,
            "XOR must be involutive: xor(xor(base, org), org) == base"
        );
    }
}

/// EC-001: two orgs that generate session IDs at the exact same nanosecond share the
/// same timestamp bytes 0–5 but must still differ in bytes 8–15.
///
/// Verified here by using the same base UUID for both XOR calls.
///
/// Traces: BC-3.2.003 invariant 4, AC-001, EC-001
#[test]
fn test_BC_3_2_003_xor_same_base_different_orgs_differ_in_bytes_8_to_15() {
    let shared_base =
        Uuid::parse_str("018e3f72-abcd-7ef0-8123-456789abcdef").expect("valid UUID literal");

    let session_a = xor_org_into_session_bytes(shared_base, org_a());
    let session_b = xor_org_into_session_bytes(shared_base, org_b());

    let bytes_a = session_a.as_bytes();
    let bytes_b = session_b.as_bytes();

    // Bytes 0-7 (timestamp + version) must be identical — only random bytes differ.
    assert_eq!(
        &bytes_a[0..8],
        &bytes_b[0..8],
        "timestamp bytes 0-7 must be unchanged by XOR"
    );

    // Bytes 8-15 must differ because org_a and org_b have different bytes there.
    assert_ne!(
        &bytes_a[8..16],
        &bytes_b[8..16],
        "random bytes 8-15 must differ between org_a and org_b sessions"
    );
}

/// EC-002: XOR with a nil OrgId (all zeros) is the identity operation — base is
/// returned unchanged.
///
/// Traces: EC-002 from S-3.2.08
#[test]
fn test_BC_3_2_003_xor_nil_org_is_identity() {
    let base = Uuid::parse_str("018e3f72-abcd-7ef0-8123-456789abcdef").expect("valid UUID literal");
    let nil_org = OrgId::from_uuid(Uuid::nil());
    let result = xor_org_into_session_bytes(base, nil_org);
    assert_eq!(
        result, base,
        "XOR with a nil OrgId must leave the base UUID unchanged (EC-002)"
    );
}

// ---------------------------------------------------------------------------
// AC-003 — generate produces a valid UUID v7 with non-zero time bits
// ---------------------------------------------------------------------------

/// The generated session ID must parse as a UUID and have version == 7 (SortRand in
/// the `uuid` crate).
///
/// Traces: AC-003, ADR-008 §2.1 D-048, BC-3.2.003 invariant 1
#[test]
fn test_BC_3_2_003_generate_produces_valid_uuid_v7() {
    let id_str = generate_crowdstrike_session_id(org_a());
    let parsed = Uuid::parse_str(&id_str)
        .unwrap_or_else(|e| panic!("session ID must be a valid UUID string, got {id_str:?}: {e}"));
    assert_eq!(
        parsed.get_version(),
        Some(Version::SortRand),
        "session ID must be UUID v7 (SortRand); got version {:?}",
        parsed.get_version()
    );
}

/// The UUID v7 time field (bytes 0-5, 48-bit millisecond timestamp) must be non-zero,
/// proving the timestamp is populated rather than defaulted.
///
/// Traces: AC-003, D-048
#[test]
fn test_BC_3_2_003_generate_uuid_time_bits_non_zero() {
    let id_str = generate_crowdstrike_session_id(org_a());
    let parsed =
        Uuid::parse_str(&id_str).unwrap_or_else(|e| panic!("session ID must parse as UUID: {e}"));
    let bytes = parsed.as_bytes();
    // Bytes 0-5 carry the 48-bit millisecond timestamp; they must be non-zero.
    let time_bytes = &bytes[0..6];
    assert_ne!(
        time_bytes, &[0u8; 6],
        "UUID v7 time field (bytes 0-5) must be non-zero (wall-clock timestamp expected)"
    );
}

// ---------------------------------------------------------------------------
// AC-004 — extract_org_id_from_session_id rejects non-prism-generated UUIDs
// ---------------------------------------------------------------------------

/// A UUID v4 string (not generated by our function) must return None.
///
/// Traces: AC-004, BC-3.2.003 postcondition 2
#[test]
fn test_BC_3_2_003_extract_rejects_uuid_v4() {
    // This is a valid UUID v4 string (version nibble = 4)
    let v4 = "550e8400-e29b-41d4-a716-446655440000";
    let result = extract_org_id_from_session_id(v4);
    assert_eq!(
        result, None,
        "extract must return None for a UUID v4 string (not v7 / not prism-generated)"
    );
}

/// A malformed (non-UUID) string must return None without panicking.
///
/// Traces: AC-004, BC-3.2.003 postcondition 2
#[test]
fn test_BC_3_2_003_extract_rejects_malformed_string() {
    let malformed = "not-a-uuid-at-all";
    let result = extract_org_id_from_session_id(malformed);
    assert_eq!(
        result, None,
        "extract must return None for a malformed non-UUID string"
    );
}

/// An empty string must return None without panicking.
///
/// Traces: AC-004
#[test]
fn test_BC_3_2_003_extract_rejects_empty_string() {
    let result = extract_org_id_from_session_id("");
    assert_eq!(result, None, "extract must return None for an empty string");
}

// ---------------------------------------------------------------------------
// AC-005 — cross-org collision impossibility (statistical, 1000 sessions each)
// ---------------------------------------------------------------------------

/// Generate 1000 session IDs for org_a and 1000 for org_b.
/// Assert no string appears in both sets (no cross-org collision).
///
/// This is a statistical impossibility check, not a proof — but 1000 UUIDs with
/// independent random bases and distinct OrgId XOR namespaces must produce disjoint
/// sets with overwhelming probability.
///
/// Traces: BC-3.2.003 postcondition 2, VP-084, AC-005, EC-001
#[test]
fn test_BC_3_2_003_cross_org_collision_impossibility_1000_each() {
    let ids_a: std::collections::HashSet<String> = (0..1000)
        .map(|_| generate_crowdstrike_session_id(org_a()))
        .collect();
    let ids_b: std::collections::HashSet<String> = (0..1000)
        .map(|_| generate_crowdstrike_session_id(org_b()))
        .collect();

    let intersection: std::collections::HashSet<&String> = ids_a.intersection(&ids_b).collect();

    assert!(
        intersection.is_empty(),
        "org_a and org_b session ID sets must be disjoint; \
        found {} collision(s): {:?}",
        intersection.len(),
        intersection.iter().take(3).collect::<Vec<_>>()
    );
}

/// Intra-org uniqueness: 1000 session IDs for the same org must all be distinct.
///
/// Traces: BC-3.2.003 invariant 1, VP-084
#[test]
fn test_BC_3_2_003_intra_org_uniqueness_1000_sessions() {
    let ids: std::collections::HashSet<String> = (0..1000)
        .map(|_| generate_crowdstrike_session_id(org_a()))
        .collect();
    assert_eq!(
        ids.len(),
        1000,
        "all 1000 session IDs for the same org must be unique"
    );
}

// ---------------------------------------------------------------------------
// VP-084 — cross-org LruCache lookup fails closed (mirrors session_registry)
// ---------------------------------------------------------------------------

/// Simulate the prism-dtu-crowdstrike session_registry lookup:
/// generate session_id_A for org_A, store it in a bare LruCache<String, _>,
/// then look up a session ID generated for org_B.  Must return None.
///
/// This validates VP-084: a token registered under org_id_A is invalid in
/// org_id_B context (D-048 structural guarantee).
///
/// Traces: VP-084, BC-3.2.003 postcondition 2, AC-002 (story), D-048
#[test]
fn test_BC_3_2_003_session_registry_lookup_org_b_misses_org_a_entry() {
    use lru::LruCache;
    use std::num::NonZeroUsize;

    let mut registry: LruCache<String, &'static str> =
        LruCache::new(NonZeroUsize::new(128).expect("non-zero capacity"));

    let session_id_a = generate_crowdstrike_session_id(org_a());
    registry.put(session_id_a, "session-data-for-org-a");

    // Generate a fresh session ID for org_b — it must not match session_id_a
    let session_id_b = generate_crowdstrike_session_id(org_b());
    let lookup = registry.get(&session_id_b);

    assert!(
        lookup.is_none(),
        "org_b session ID must not resolve org_a's registry entry (VP-084)"
    );
}

// ---------------------------------------------------------------------------
// XOR byte-level correctness — bytes 0-7 preserved, bytes 8-15 XORed
// ---------------------------------------------------------------------------

/// Verify that xor_org_into_session_bytes preserves bytes 0-7 exactly and
/// only modifies bytes 8-15.
///
/// Traces: BC-3.2.003 invariant 4, ADR-008 §2.1 (version/variant bits untouched)
#[test]
fn test_BC_3_2_003_xor_preserves_bytes_0_to_7_modifies_8_to_15() {
    let base = Uuid::parse_str("018e3f72-abcd-7ef0-8123-456789abcdef").expect("valid UUID literal");
    let result = xor_org_into_session_bytes(base, org_a());

    let base_bytes = base.as_bytes();
    let result_bytes = result.as_bytes();

    // Bytes 0-7 (timestamp + version nibble + variant) must be unchanged
    assert_eq!(
        &base_bytes[0..8],
        &result_bytes[0..8],
        "bytes 0-7 (timestamp, version, variant) must be preserved by XOR"
    );

    // Bytes 8-15 must be the XOR of base[8..16] with org_a's bytes[8..16]
    let org_bytes = org_a().as_uuid().into_bytes();
    for i in 8..16 {
        let expected = base_bytes[i] ^ org_bytes[i];
        assert_eq!(
            result_bytes[i], expected,
            "byte {i} of result must be base[{i}] XOR org[{i}]"
        );
    }
}
