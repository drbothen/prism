//! RNG-free stable per-record offset fold for seeded-deterministic timestamp
//! derivation (review-2026-06-10 P1-02; lifted from the CrowdStrike generator
//! to avoid triplication across the Cyberint/Claroty/Armis generators).
//!
//! INVARIANT (INV-SECONDARY-RNG-STREAM-INDEPENDENCE-001): this fold MUST stay
//! RNG-free. Per-record timestamp variance derives from hashing the record key
//! with the seed — it never draws from the primary ChaCha20 stream, so adding
//! or reordering timestamp fields cannot perturb the byte-identity of other
//! generated values (BC-3.4.001 / AC-012).

/// Stable per-record offset fold.
///
/// NOT `std::hash::DefaultHasher` — its output is explicitly unstable across
/// Rust releases, which would break BC-3.4.001 byte-identical determinism.
/// Simple FNV-1a-style fold over the record-key bytes mixed with the seed.
pub fn stable_offset(record_key: &str, seed: u64) -> u64 {
    let mut acc: u64 = 0xcbf2_9ce4_8422_2325 ^ seed;
    for b in record_key.as_bytes() {
        acc ^= u64::from(*b);
        acc = acc.wrapping_mul(0x0000_0100_0000_01b3);
    }
    acc
}

#[cfg(test)]
mod tests {
    use super::stable_offset;

    /// Deterministic: same inputs always fold to the same value.
    #[test]
    fn test_stable_offset_deterministic() {
        assert_eq!(
            stable_offset("dev-deadbeef-42-0", 42),
            stable_offset("dev-deadbeef-42-0", 42)
        );
    }

    /// Distinct record keys fold to distinct values (variance source for
    /// per-record timestamps — P1-02 window discrimination).
    #[test]
    fn test_stable_offset_varies_by_key() {
        assert_ne!(
            stable_offset("dev-deadbeef-42-0", 42),
            stable_offset("dev-deadbeef-42-1", 42)
        );
    }

    /// Distinct seeds fold to distinct values for the same key.
    #[test]
    fn test_stable_offset_varies_by_seed() {
        assert_ne!(
            stable_offset("dev-deadbeef-42-0", 1),
            stable_offset("dev-deadbeef-42-0", 2)
        );
    }

    /// Pinned value: guards against accidental algorithm drift, which would
    /// silently change every derived timestamp across the DTU fleet.
    #[test]
    fn test_stable_offset_pinned_vector() {
        // FNV-1a offset basis ^ 0, folded over "a" (0x61):
        // (0xcbf29ce484222325 ^ 0x61) * 0x100000001b3 (wrapping)
        let expected = (0xcbf2_9ce4_8422_2325_u64 ^ 0x61).wrapping_mul(0x0000_0100_0000_01b3);
        assert_eq!(stable_offset("a", 0), expected);
    }
}
