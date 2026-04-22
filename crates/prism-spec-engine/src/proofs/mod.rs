//! Formal verification proof harnesses for prism-spec-engine.
//!
//! - `spec_validator`: VP-059 proptest harness — all-errors-collected, no fail-fast.
//! - `infusion_spec`: VP-048 Kani proof — N fields produces exactly N UDF descriptors.
//! - `infusion_dedup`: VP-049 proptest — source calls equal unique value count.
//! - `plugin_linker`: VP-040 Kani/proptest — linker excludes all WASI namespace imports.
//! - `plugin_memory`: VP-041 proptest — memory limit boundary: at-limit succeeds, over-limit traps.
//! - `plugin_hot_reload`: VP-042 proptest — failed compile retains old InstancePre.
//! - `plugin_wit_validation`: VP-043 proptest — WIT validation rejects missing exports.

pub mod infusion_dedup;
pub mod infusion_spec;
pub mod spec_validator;
pub mod plugin_linker;
pub mod plugin_memory;
pub mod plugin_hot_reload;
pub mod plugin_wit_validation;
