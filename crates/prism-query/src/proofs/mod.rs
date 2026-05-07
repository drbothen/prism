//! Formal verification proof harnesses for prism-query.
//!
//! All Kani proofs are gated with `#[cfg(kani)]` and have zero effect on
//! test or release builds.
//!
//! Proptest proofs (VP-031) run under `#[cfg(test)]` and are RED by design
//! until S-3.02 implementation is complete (todo!() bodies).
//!
//! | Proof module        | VP ID  | Method   | Property                                        |
//! |---------------------|--------|----------|-------------------------------------------------|
//! | `vp014_size_limit`  | VP-014 | Kani     | Queries > MAX_QUERY_SIZE always return Err      |
//! | `vp015_depth_limit` | VP-015 | Kani     | Nesting depth > 64 always returns Err           |
//! | `vp031_pushdown`    | VP-031 | Proptest | REQUIRED columns always produce PushDown        |
//!
//! ## Canonical Kani invocation flags (Section D harmonization)
//!
//! All harnesses in this crate use `--no-unwinding-checks --default-unwind 2`.
//! These flags are required because `effective_*_limit()` functions call
//! `std::env::var(...)` whose internal loops (e.g. `memchr_naive`) Kani cannot
//! bound without disabling unwinding assertions. The property assertions still
//! verify soundly — only the meta-check "all loops fully unrolled" is skipped.
//!
//! Standard invocation template:
//! ```text
//! cargo kani -p prism-query \
//!     --harness "proofs::<module>::kani_proofs::<harness_name>" \
//!     --exact --no-unwinding-checks --default-unwind 2
//! ```
//!
//! Story: S-3.01 (VP-014, VP-015) | S-3.02 (VP-031)

// The `#[cfg(test)]` fallback tests in vp014_size_limit and vp015_depth_limit
// use `result.unwrap_err()` in assertion contexts where a panic on Ok would
// correctly indicate a failing property test. The unwrap is intentional.
#[allow(clippy::unwrap_used)]
pub mod vp014_size_limit;
#[allow(clippy::unwrap_used)]
pub mod vp015_depth_limit;

// VP-031: REQUIRED columns always produce PushDown (BC-2.11.007, S-3.02).
// RED by design — todo!() bodies fire until implementation is complete.
pub mod vp031_pushdown;

// VP-025: Cache key derivation is deterministic (BC-2.07.005, S-3.05).
// RED by design — todo!() bodies in derive_push_down_hash fire until S-3.05 is implemented.
pub mod vp025_cache_key;
