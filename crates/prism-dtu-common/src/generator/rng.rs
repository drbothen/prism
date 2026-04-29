//! Deterministic RNG initialisation (BC-3.4.001 invariant 2).
//!
//! INVARIANT: NEVER call `rand::thread_rng()`, `SystemTime::now()`, or any
//! non-deterministic entropy source anywhere in this module or its callers.
//! CI grep check enforces this (VP-3.4.001-D).

use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;

use super::fixture::OrgId;

/// Construct a deterministic `ChaCha20Rng` from a seed and an `OrgId`.
///
/// Formula (BC-3.4.001 invariant 2):
/// ```text
/// org_id_hash = u64::from_le_bytes(org_id.as_bytes()[0..8])
/// ChaCha20Rng::seed_from_u64(seed ^ org_id_hash)
/// ```
///
/// This function MUST NOT call `rand::thread_rng()`, `SystemTime::now()`, or
/// any other non-deterministic entropy source (ADR-009 §3.2).
pub fn seeded_rng(seed: u64, org_id: &OrgId) -> ChaCha20Rng {
    // org_id_hash: first 8 bytes of UUID interpreted as little-endian u64 (BC-3.4.001 invariant 2).
    // SAFETY: OrgId is always [u8; 16] — slicing [0..8] is always valid; copy to array directly.
    let all = org_id.as_bytes();
    let bytes: [u8; 8] = [
        all[0], all[1], all[2], all[3], all[4], all[5], all[6], all[7],
    ];
    let org_id_hash = u64::from_le_bytes(bytes);
    ChaCha20Rng::seed_from_u64(seed ^ org_id_hash)
}
