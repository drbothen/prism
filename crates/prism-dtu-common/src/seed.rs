//! Deterministic seeded RNG utilities. NEVER use `rand::thread_rng()` in DTU crates.

use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;

/// Construct a deterministic [`ChaCha20Rng`] from the given seed.
///
/// All randomness in DTU stubs MUST flow through this function to guarantee
/// reproducible test runs (AC-5).
pub fn seeded_rng(seed: u64) -> ChaCha20Rng {
    ChaCha20Rng::seed_from_u64(seed)
}
