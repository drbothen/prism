//! Deterministic RNG initialisation (BC-3.4.001 invariant 2).
//!
//! INVARIANT: NEVER call `rand::thread_rng()`, `SystemTime::now()`, or any
//! non-deterministic entropy source anywhere in this module or its callers.
//! CI grep check enforces this (VP-3.4.001-D).

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
pub fn seeded_rng(_seed: u64, _org_id: &OrgId) -> ChaCha20Rng {
    todo!()
}
