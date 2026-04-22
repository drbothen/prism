# AC-5: seeded_rng — same seed produces identical sequence

## Acceptance Criterion

Given `seeded_rng(42)` is called twice, When any random value sequence
is drawn from each RNG, Then both sequences are identical (deterministic, same seed).

## Test

- File: `crates/prism-dtu-common/tests/ac_5_seeded_rng_determinism.rs`
- Functions: `ac_5_seeded_rng_same_seed_produces_identical_sequence`, `ac_5_seeded_rng_different_seeds_produce_different_sequences`
- Test command: `cargo test --features prism-dtu-common/dtu --test ac_5_seeded_rng_determinism`

## Implementation (excerpt)

File: `crates/prism-dtu-common/src/seed.rs`

```rust
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;

/// Construct a deterministic [`ChaCha20Rng`] from the given seed.
///
/// All randomness in DTU stubs MUST flow through this function to guarantee
/// reproducible test runs (AC-5).
pub fn seeded_rng(seed: u64) -> ChaCha20Rng {
    ChaCha20Rng::seed_from_u64(seed)
}
```

## Test output

```
running 2 tests
test ac_5_seeded_rng_same_seed_produces_identical_sequence ... ok
test ac_5_seeded_rng_different_seeds_produce_different_sequences ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Mapping

`ChaCha20Rng` is a cryptographically-strong CSPRNG with guaranteed determinism when seeded via `seed_from_u64`; the test draws 10 `u32` values from two independently-seeded instances with seed 42 and asserts all 10 values match, then confirms different seeds produce different sequences.
