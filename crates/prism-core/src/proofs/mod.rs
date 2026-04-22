//! Kani formal verification proofs for prism-core invariants.
//!
//! All proofs are gated with `#[cfg(kani)]` and have zero effect on
//! test or release builds.

pub mod tenant_id;
