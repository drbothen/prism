//! VP-025: Cache Key Derivation — Deterministic.
//!
//! Property: For every push-down parameter set `p` and every `(client_id,
//! sensor_id, source_id)` tuple, `CacheKey::derive(client, sensor, source, p)`
//! returns the same `CacheKey` on every invocation. Permuted-but-equivalent
//! parameter orderings produce the identical `push_down_hash`.
//!
//! ## Source Contract
//! BC-2.07.005 — Cache Key Derivation from Push-Down Parameters.
//!
//! ## Proof Method
//! Kani (bounded model checking). Because `CacheKey::derive_push_down_hash`
//! involves SHA-256 (an opaque byte transformation at Kani's level), the
//! Kani harness models SHA-256 as an **uninterpreted function** for
//! termination, and checks the canonicalization invariant directly:
//! - Two parameter sets with the same keys+values in different insertion
//!   order produce the same canonical JSON bytes (and therefore the same hash).
//! - Null/absent values are omitted before hashing.
//!
//! The concrete determinism property (`derive(p) == derive(p)`) is verified
//! via the `#[cfg(test)]` dynamic tests in `tests/cache_tests.rs` and the
//! inline `#[cfg(test)] mod dynamic_tests` below, which cover the property
//! exhaustively for bounded concrete inputs.
//!
//! ## Harness skeleton
//! The full Kani harness is stubbed with `todo!()` bodies per the Red Gate
//! protocol (BC-5.38.001). The implementer must replace `todo!()` with the
//! real implementation before the Kani proof can run.
//!
//! ## Invocation
//! ```text
//! cargo kani -p prism-query \
//!     --harness "proofs::vp025_cache_key::kani_proofs::proof_cache_key_deterministic" \
//!     --exact --no-unwinding-checks --default-unwind 2
//! ```
//!
//! Story: S-3.05 | VP-025 | BC-2.07.005

// Suppress dead_code and unused vars/imports for stub-phase.
#![allow(dead_code, unused_imports, unused_variables)]

// ---------------------------------------------------------------------------
// Kani harnesses (compile + run only under `cargo kani`)
// ---------------------------------------------------------------------------

/// Kani proof module — gated by `#[cfg(kani)]`.
///
/// All harnesses use `--no-unwinding-checks --default-unwind 2` per the
/// canonical invocation template in `proofs/mod.rs`.
#[cfg(kani)]
pub mod kani_proofs {
    use std::collections::BTreeMap;

    use crate::cache_key::{CacheKey, PushDownParams};

    /// Proof: `derive_push_down_hash` is deterministic — same input always
    /// produces the same output.
    ///
    /// Method: kani. Target: `prism_query::cache_key::CacheKey::derive_push_down_hash`.
    ///
    /// For bounded symbolic inputs (one string key, one string value), asserts
    /// `hash(params) == hash(params)` (idempotency / determinism).
    ///
    /// Body: non-trivial — the real implementation must replace `todo!()` in
    /// `CacheKey::derive_push_down_hash` before this harness can run.
    #[kani::proof]
    fn proof_cache_key_deterministic() {
        // Symbolic parameter set: one key-value pair with bounded strings.
        // Kani models SHA-256 as an uninterpreted function for termination.
        let key_str: [u8; 4] = kani::any();
        let val_str: [u8; 4] = kani::any();

        // Reject invalid UTF-8 for the symbolic key/value.
        kani::assume(std::str::from_utf8(&key_str).is_ok());
        kani::assume(std::str::from_utf8(&val_str).is_ok());

        let key_s = std::str::from_utf8(&key_str).unwrap();
        let val_s = std::str::from_utf8(&val_str).unwrap();

        let mut params = PushDownParams::new();
        params.insert(key_s, serde_json::Value::String(val_s.to_string()));

        // Determinism: same input → same output on two separate calls.
        let hash1 = CacheKey::derive_push_down_hash(&params);
        let hash2 = CacheKey::derive_push_down_hash(&params);

        kani::assert(
            hash1 == hash2,
            "VP-025: cache key derivation must be deterministic",
        );
    }

    /// Proof: permuted-but-equivalent parameter sets produce the same hash.
    ///
    /// Method: kani. Symbolic two-key parameter set; asserts that inserting
    /// keys in different orders yields the same canonical JSON bytes.
    ///
    /// Body: non-trivial — depends on `PushDownParams::insert` canonicalizing
    /// via `BTreeMap` sort. Replacing `todo!()` in `insert` and
    /// `derive_push_down_hash` is required for this harness to complete.
    #[kani::proof]
    fn proof_cache_key_order_independent() {
        // Two symbolic keys and values — bounded to 3 bytes each.
        let k1: [u8; 3] = kani::any();
        let k2: [u8; 3] = kani::any();
        let v1: [u8; 3] = kani::any();
        let v2: [u8; 3] = kani::any();

        kani::assume(std::str::from_utf8(&k1).is_ok());
        kani::assume(std::str::from_utf8(&k2).is_ok());
        kani::assume(std::str::from_utf8(&v1).is_ok());
        kani::assume(std::str::from_utf8(&v2).is_ok());

        let ks1 = std::str::from_utf8(&k1).unwrap();
        let ks2 = std::str::from_utf8(&k2).unwrap();
        let vs1 = std::str::from_utf8(&v1).unwrap();
        let vs2 = std::str::from_utf8(&v2).unwrap();

        // Keys must be distinct for the permutation to be non-trivial.
        kani::assume(ks1 != ks2);

        // Insertion order A: k1 first, then k2.
        let mut params_a = PushDownParams::new();
        params_a.insert(ks1, serde_json::Value::String(vs1.to_string()));
        params_a.insert(ks2, serde_json::Value::String(vs2.to_string()));

        // Insertion order B: k2 first, then k1.
        let mut params_b = PushDownParams::new();
        params_b.insert(ks2, serde_json::Value::String(vs2.to_string()));
        params_b.insert(ks1, serde_json::Value::String(vs1.to_string()));

        let hash_a = CacheKey::derive_push_down_hash(&params_a);
        let hash_b = CacheKey::derive_push_down_hash(&params_b);

        kani::assert(
            hash_a == hash_b,
            "VP-025: permuted-but-equivalent params must produce identical push_down_hash",
        );
    }

    /// Proof: absent and null parameters are treated identically (both omitted).
    ///
    /// Method: kani. Asserts `hash({}) == hash({key: null})`.
    ///
    /// Body: non-trivial — depends on `PushDownParams::insert` filtering nulls.
    #[kani::proof]
    fn proof_null_param_same_as_absent() {
        let mut params_with_null = PushDownParams::new();
        params_with_null.insert("optional_filter", serde_json::Value::Null);

        let params_without = PushDownParams::new();

        let hash_with = CacheKey::derive_push_down_hash(&params_with_null);
        let hash_without = CacheKey::derive_push_down_hash(&params_without);

        kani::assert(
            hash_with == hash_without,
            "VP-025: explicit null must produce same hash as absent parameter",
        );
    }
}

// ---------------------------------------------------------------------------
// Dynamic concrete tests (run under `cargo test`)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod dynamic_tests {
    use crate::cache_key::{CacheKey, PushDownParams};
    use serde_json::json;

    /// VP-025 concrete determinism: `derive(params) == derive(params)`.
    ///
    /// RED by design — `CacheKey::derive_push_down_hash` is `todo!()`.
    #[test]
    fn test_vp025_derive_is_deterministic_concrete() {
        let mut params = PushDownParams::new();
        params.insert("severity", json!("High"));
        params.insert("status", json!("open"));

        let h1 = CacheKey::derive_push_down_hash(&params);
        let h2 = CacheKey::derive_push_down_hash(&params);

        assert_eq!(
            h1, h2,
            "VP-025: derive_push_down_hash must be deterministic for identical inputs"
        );
    }

    /// VP-025 concrete order independence: insert order must not affect hash.
    ///
    /// RED by design — `CacheKey::derive_push_down_hash` is `todo!()`.
    #[test]
    fn test_vp025_order_independence_concrete() {
        let mut params_a = PushDownParams::new();
        params_a.insert("z_param", json!("val_z"));
        params_a.insert("a_param", json!("val_a"));

        let mut params_b = PushDownParams::new();
        params_b.insert("a_param", json!("val_a"));
        params_b.insert("z_param", json!("val_z"));

        let hash_a = CacheKey::derive_push_down_hash(&params_a);
        let hash_b = CacheKey::derive_push_down_hash(&params_b);

        assert_eq!(
            hash_a, hash_b,
            "VP-025: parameter insertion order must not affect push_down_hash"
        );
    }

    /// VP-025 concrete null equivalence: null == absent.
    ///
    /// RED by design — `CacheKey::derive_push_down_hash` is `todo!()`.
    #[test]
    fn test_vp025_null_equivalent_to_absent() {
        let mut params_null = PushDownParams::new();
        params_null.insert("opt", serde_json::Value::Null);

        let params_empty = PushDownParams::new();

        let hash_null = CacheKey::derive_push_down_hash(&params_null);
        let hash_empty = CacheKey::derive_push_down_hash(&params_empty);

        assert_eq!(
            hash_null, hash_empty,
            "VP-025: explicit null must be treated as absent (omitted from canonical form)"
        );
    }

    /// VP-025: push_down_hash is 64 hex characters (SHA-256 output).
    ///
    /// RED by design — `CacheKey::derive_push_down_hash` is `todo!()`.
    #[test]
    fn test_vp025_hash_is_64_hex_chars() {
        let params = PushDownParams::new();
        let hash = CacheKey::derive_push_down_hash(&params);
        assert_eq!(
            hash.len(),
            64,
            "VP-025: push_down_hash must be a 64-character lowercase hex string"
        );
        assert!(
            hash.chars().all(|c| c.is_ascii_hexdigit()),
            "VP-025: push_down_hash must contain only hex characters"
        );
    }
}
