// AC-5: seeded_rng(seed) produces identical sequences for the same seed.
//
// Expected failure mode: todo!("implement seeded_rng per AC-5") panics at runtime.

use prism_dtu_common::seeded_rng;
use rand::RngCore;

#[test]
fn ac_5_seeded_rng_same_seed_produces_identical_sequence() {
    let mut rng1 = seeded_rng(42);
    let mut rng2 = seeded_rng(42);

    let seq1: Vec<u32> = (0..10).map(|_| rng1.next_u32()).collect();
    let seq2: Vec<u32> = (0..10).map(|_| rng2.next_u32()).collect();

    assert_eq!(
        seq1, seq2,
        "AC-5: same seed must produce identical RNG sequence"
    );
}

#[test]
fn ac_5_seeded_rng_different_seeds_produce_different_sequences() {
    let mut rng_a = seeded_rng(1);
    let mut rng_b = seeded_rng(2);

    let seq_a: Vec<u32> = (0..10).map(|_| rng_a.next_u32()).collect();
    let seq_b: Vec<u32> = (0..10).map(|_| rng_b.next_u32()).collect();

    assert_ne!(
        seq_a, seq_b,
        "AC-5: different seeds must produce different sequences"
    );
}
