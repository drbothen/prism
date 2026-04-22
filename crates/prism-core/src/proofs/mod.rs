//! Kani formal verification proofs for prism-core invariants.
//!
//! All proofs are gated with `#[cfg(kani)]` and have zero effect on
//! test or release builds.

// S-1.01 proofs
pub mod tenant_id;

// S-1.02 proofs
pub mod case_status;
pub mod case_status_exhaustive;
pub mod credential_name;
pub mod cursor;

// S-1.03 proofs
#[cfg(kani)]
pub mod capability;
