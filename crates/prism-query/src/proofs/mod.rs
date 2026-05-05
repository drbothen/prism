//! Kani formal verification proof harnesses for prism-query.
//!
//! All proofs are gated with `#[cfg(kani)]` and have zero effect on
//! test or release builds.
//!
//! | Proof module      | VP ID  | Property                                        |
//! |-------------------|--------|-------------------------------------------------|
//! | `vp014_size_limit`| VP-014 | Queries > MAX_QUERY_SIZE always return Err      |
//! | `vp015_depth_limit`| VP-015| Nesting depth > 64 always returns Err           |
//!
//! Story: S-3.01

pub mod vp014_size_limit;
pub mod vp015_depth_limit;
