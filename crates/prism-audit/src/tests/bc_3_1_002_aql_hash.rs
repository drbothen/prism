//! Tests for BC-3.1.002 (TD-ADR005-002 fix) — SHA-256 `aql_hash` computation.
//!
//! BC-3.1.002 invariant 1 / TD-ADR005-002 postconditions tested:
//!   - `compute_aql_hash` returns a 64-character lowercase hex string (SHA-256 digest).
//!   - The hash is deterministic: identical input → identical output.
//!   - The hash is one-way: the raw query string is NOT recoverable from the hash.
//!   - `compute_aql_hash("")` produces the canonical SHA-256 of the empty string.
//!   - `compute_aql_hash` accepts very long (> 65,535 byte) query strings.
//!   - `compute_aql_hash` accepts Unicode query strings.
//!   - Two distinct inputs produce distinct hashes (collision resistance sanity check).
//!
//! AC-6 (S-3.1.07): `aql_hash("SELECT * FROM crowdstrike.devices")` equals the
//! expected SHA-256 hex digest.
//!
//! Test naming: `test_BC_3_1_002_<assertion_name>()` per factory naming convention.
//!
//! ALL tests in this file exercise `AuditEntry::compute_aql_hash`, which currently
//! contains `todo!()`. Every test must PANIC at the Red Gate — this is the expected
//! Red Gate behavior for S-3.1.07.

use crate::audit_entry::AuditEntry;

// ── Canonical test vectors (AC-6 / BC-3.1.002 invariant 1) ───────────────────
//
// SHA-256("SELECT * FROM crowdstrike.devices") =
//   2e99758548972a8e8822ad47fa1017ff72f06f3ff6a016851f45c398732bc50c
//
// Computed externally: echo -n "SELECT * FROM crowdstrike.devices" | sha256sum
// (verified against RFC 6234 SHA-256 test vectors).
//
// SHA-256("") =
//   e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855

const CROWDSTRIKE_QUERY: &str = "SELECT * FROM crowdstrike.devices";
const CROWDSTRIKE_HASH: &str = "2e99758548972a8e8822ad47fa1017ff72f06f3ff6a016851f45c398732bc50c";

const EMPTY_STRING_HASH: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

// ── AC-6: canonical golden test vector ───────────────────────────────────────

/// AC-6 (S-3.1.07) / BC-3.1.002 invariant 1:
/// `compute_aql_hash("SELECT * FROM crowdstrike.devices")` must equal the
/// exact SHA-256 hex digest for that string.
///
/// This test will PANIC at the Red Gate (todo!() stub).
#[test]
fn test_BC_3_1_002_aql_hash_canonical_crowdstrike_query() {
    let result = AuditEntry::compute_aql_hash(CROWDSTRIKE_QUERY);
    assert_eq!(
        result, CROWDSTRIKE_HASH,
        "AC-6: aql_hash({CROWDSTRIKE_QUERY:?}) must equal the SHA-256 hex digest '{CROWDSTRIKE_HASH}'"
    );
}

// ── Format: 64-char lowercase hex ────────────────────────────────────────────

/// BC-3.1.002 invariant 1: `compute_aql_hash` must return exactly 64 lowercase
/// hexadecimal characters (SHA-256 produces a 32-byte / 256-bit digest → 64 hex chars).
///
/// This test will PANIC at the Red Gate (todo!() stub).
#[test]
fn test_BC_3_1_002_aql_hash_is_64_char_hex_string() {
    let result = AuditEntry::compute_aql_hash("SELECT * FROM armis.devices LIMIT 10");

    assert_eq!(
        result.len(),
        64,
        "BC-3.1.002 invariant 1: aql_hash must be exactly 64 characters (SHA-256 hex digest), got {} chars",
        result.len()
    );

    assert!(
        result
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_uppercase()),
        "BC-3.1.002 invariant 1: aql_hash must be lowercase hex, got: '{result}'"
    );
}

/// BC-3.1.002 invariant 1: the hash must be lowercase (not uppercase hex).
/// Some implementations produce uppercase; SHA-256 convention here is lowercase.
///
/// This test will PANIC at the Red Gate (todo!() stub).
#[test]
fn test_BC_3_1_002_aql_hash_is_lowercase_hex() {
    let result = AuditEntry::compute_aql_hash("SELECT * FROM crowdstrike.alerts");

    assert_eq!(
        result,
        result.to_lowercase(),
        "BC-3.1.002 invariant 1: aql_hash must be lowercase hex (no uppercase hex digits)"
    );
}

// ── Determinism: same input → same output ────────────────────────────────────

/// BC-3.1.002 invariant 1 / EC-006 (TD-ADR005-002 fix):
/// `compute_aql_hash` must be deterministic — calling it twice with the same
/// input must produce identical output (replaces DefaultHasher which was not
/// deterministic across processes).
///
/// This test will PANIC at the Red Gate (todo!() stub).
#[test]
fn test_BC_3_1_002_aql_hash_is_deterministic_same_call() {
    let query = "SELECT * FROM armis.devices WHERE criticality = 'HIGH'";
    let hash1 = AuditEntry::compute_aql_hash(query);
    let hash2 = AuditEntry::compute_aql_hash(query);

    assert_eq!(
        hash1, hash2,
        "BC-3.1.002 invariant 1: compute_aql_hash must be deterministic — two calls with the same input must produce the same hash"
    );
}

/// BC-3.1.002 invariant 1: determinism holds across different local-variable
/// bindings (verifies there is no hidden mutable state in the hash computation).
///
/// This test will PANIC at the Red Gate (todo!() stub).
#[test]
fn test_BC_3_1_002_aql_hash_deterministic_independent_invocations() {
    let query = CROWDSTRIKE_QUERY;

    let h1 = AuditEntry::compute_aql_hash(query);
    let h2 = AuditEntry::compute_aql_hash(query);
    let h3 = AuditEntry::compute_aql_hash(query);

    assert_eq!(
        h1, h2,
        "BC-3.1.002: hash must be identical on invocation 1 vs 2"
    );
    assert_eq!(
        h2, h3,
        "BC-3.1.002: hash must be identical on invocation 2 vs 3"
    );
    assert_eq!(
        h1, CROWDSTRIKE_HASH,
        "BC-3.1.002 AC-6: hash must equal the canonical expected digest"
    );
}

// ── Privacy: hash is one-way ──────────────────────────────────────────────────

/// Privacy / BC-3.1.002 invariant 1: the aql_hash must NOT contain the raw
/// query string. It is a one-way cryptographic digest.
///
/// We assert that the 64-char hex output does not contain the input as a
/// substring — i.e., the raw query string is not leaked into the hash value.
///
/// This test will PANIC at the Red Gate (todo!() stub).
#[test]
fn test_BC_3_1_002_aql_hash_does_not_contain_raw_query() {
    let query = "SELECT * FROM crowdstrike.devices";
    let hash = AuditEntry::compute_aql_hash(query);

    assert!(
        !hash.contains(query),
        "Privacy: aql_hash must not contain the raw query string — it is a one-way digest. Hash: '{hash}'"
    );

    // Also verify the hash doesn't contain any easily recognizable token from the query.
    assert!(
        !hash.contains("SELECT"),
        "Privacy: aql_hash must not contain the literal 'SELECT' from the input query"
    );
    assert!(
        !hash.contains("crowdstrike"),
        "Privacy: aql_hash must not contain the literal 'crowdstrike' from the input query"
    );
}

// ── Edge case: empty query ────────────────────────────────────────────────────

/// BC-3.1.002 edge case: `compute_aql_hash("")` must return the canonical
/// SHA-256 digest of the empty string (not panic, not return an empty string).
///
/// SHA-256("") = e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
///
/// This test will PANIC at the Red Gate (todo!() stub).
#[test]
fn test_BC_3_1_002_aql_hash_empty_query_returns_sha256_of_empty_string() {
    let hash = AuditEntry::compute_aql_hash("");

    assert_eq!(
        hash, EMPTY_STRING_HASH,
        "BC-3.1.002 edge case: aql_hash(\"\") must equal SHA-256 of empty string '{EMPTY_STRING_HASH}'"
    );

    // Must still be 64 chars.
    assert_eq!(
        hash.len(),
        64,
        "BC-3.1.002: aql_hash of empty string must still be 64 hex chars"
    );
}

/// BC-3.1.002 edge case: `compute_aql_hash("")` must not return an empty string.
///
/// This test will PANIC at the Red Gate (todo!() stub).
#[test]
fn test_BC_3_1_002_aql_hash_empty_query_does_not_return_empty_string() {
    let hash = AuditEntry::compute_aql_hash("");

    assert!(
        !hash.is_empty(),
        "BC-3.1.002 edge case: aql_hash of empty query must not itself be empty — SHA-256 always produces 64 hex chars"
    );
}

// ── Edge case: very long query ────────────────────────────────────────────────

/// BC-3.1.002 edge case: `compute_aql_hash` must not panic or truncate on
/// a very long query string (> 65,535 bytes). SHA-256 handles arbitrary-length input.
///
/// This test will PANIC at the Red Gate (todo!() stub).
#[test]
fn test_BC_3_1_002_aql_hash_very_long_query_does_not_panic() {
    // 100,000 ASCII characters — well beyond any practical PrismQL query.
    let long_query = "SELECT * FROM armis.devices ".repeat(3572); // ~100,016 chars
    assert!(
        long_query.len() > 65_535,
        "precondition: query must be > 65535 bytes"
    );

    let hash = AuditEntry::compute_aql_hash(&long_query);

    assert_eq!(
        hash.len(),
        64,
        "BC-3.1.002 edge case: aql_hash of very long query must still be 64 hex chars (SHA-256 is length-independent)"
    );
    assert!(
        hash.chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_uppercase()),
        "BC-3.1.002 edge case: aql_hash of very long query must be lowercase hex"
    );
}

/// BC-3.1.002 edge case: two different long queries produce different hashes.
///
/// This test will PANIC at the Red Gate (todo!() stub).
#[test]
fn test_BC_3_1_002_aql_hash_distinct_long_queries_produce_distinct_hashes() {
    let query_a = "SELECT a FROM t ".repeat(4000);
    let query_b = "SELECT b FROM t ".repeat(4000);

    let hash_a = AuditEntry::compute_aql_hash(&query_a);
    let hash_b = AuditEntry::compute_aql_hash(&query_b);

    assert_ne!(
        hash_a, hash_b,
        "BC-3.1.002 edge case: two distinct long queries must produce distinct hashes"
    );
}

// ── Edge case: unicode query ──────────────────────────────────────────────────

/// BC-3.1.002 edge case: `compute_aql_hash` accepts Unicode input (e.g., org
/// names, sensor labels, or analyst comments in non-ASCII scripts).
///
/// This test will PANIC at the Red Gate (todo!() stub).
#[test]
fn test_BC_3_1_002_aql_hash_unicode_query_produces_64_char_hex() {
    // Greek, CJK, emoji, Arabic — all valid UTF-8.
    let unicode_query = "SELECT * FROM sensors WHERE label = 'αβγ 中文 🔒 العربية'";

    let hash = AuditEntry::compute_aql_hash(unicode_query);

    assert_eq!(
        hash.len(),
        64,
        "BC-3.1.002 edge case: aql_hash of unicode query must be 64 hex chars"
    );
    assert!(
        hash.chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_uppercase()),
        "BC-3.1.002 edge case: aql_hash of unicode query must be lowercase hex"
    );
}

/// BC-3.1.002 edge case: unicode and ASCII variants of the same semantic query
/// must produce distinct hashes (byte-level determinism, not semantic equivalence).
///
/// This test will PANIC at the Red Gate (todo!() stub).
#[test]
fn test_BC_3_1_002_aql_hash_unicode_vs_ascii_are_distinct() {
    let ascii_query = "SELECT * FROM sensors WHERE label = 'abc'";
    let unicode_query = "SELECT * FROM sensors WHERE label = 'αβγ'";

    let hash_ascii = AuditEntry::compute_aql_hash(ascii_query);
    let hash_unicode = AuditEntry::compute_aql_hash(unicode_query);

    assert_ne!(
        hash_ascii, hash_unicode,
        "BC-3.1.002 edge case: ASCII and Unicode queries must produce different hashes (byte-level digest)"
    );
}

// ── Collision resistance: distinct inputs → distinct hashes ──────────────────

/// BC-3.1.002 / collision resistance sanity check: two distinct query strings
/// must produce distinct SHA-256 hashes. (SHA-256 collision resistance is a
/// cryptographic property; this test documents the requirement.)
///
/// This test will PANIC at the Red Gate (todo!() stub).
#[test]
fn test_BC_3_1_002_aql_hash_distinct_inputs_produce_distinct_hashes() {
    let pairs = [
        (
            "SELECT * FROM crowdstrike.devices",
            "SELECT * FROM armis.devices",
        ),
        (
            "SELECT * FROM crowdstrike.alerts WHERE severity = 'HIGH'",
            "SELECT * FROM crowdstrike.alerts WHERE severity = 'LOW'",
        ),
        (
            "", // empty
            "SELECT 1",
        ),
        (
            "a", // single-char inputs
            "b",
        ),
    ];

    for (q1, q2) in pairs {
        let h1 = AuditEntry::compute_aql_hash(q1);
        let h2 = AuditEntry::compute_aql_hash(q2);
        assert_ne!(
            h1, h2,
            "BC-3.1.002 collision sanity: aql_hash({q1:?}) == aql_hash({q2:?}) — distinct inputs must produce distinct hashes"
        );
    }
}

// ── Prefix sensitivity: single-byte difference changes hash ──────────────────

/// BC-3.1.002 / avalanche effect sanity: changing a single character in the
/// query must produce a completely different hash (SHA-256 avalanche property).
///
/// This test will PANIC at the Red Gate (todo!() stub).
#[test]
fn test_BC_3_1_002_aql_hash_single_byte_change_produces_different_hash() {
    let base = "SELECT * FROM crowdstrike.devices WHERE id = 'AAAA'";
    let mutated = "SELECT * FROM crowdstrike.devices WHERE id = 'AAAB'";

    let h_base = AuditEntry::compute_aql_hash(base);
    let h_mutated = AuditEntry::compute_aql_hash(mutated);

    assert_ne!(
        h_base, h_mutated,
        "BC-3.1.002 avalanche: a single-byte change in the query must produce a different SHA-256 hash"
    );
}
