//! Formal verification proof harnesses for prism-spec-engine.
//!
//! - `spec_validator`: VP-059 proptest harness — all-errors-collected, no fail-fast.
//! - `infusion_spec`: VP-048 Kani proof — N fields produces exactly N UDF descriptors.
//! - `infusion_dedup`: VP-049 proptest — source calls equal unique value count.

pub mod infusion_dedup;
pub mod infusion_spec;
pub mod spec_validator;
