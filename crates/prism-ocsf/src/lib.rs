//! `prism-ocsf` — OCSF normalization infrastructure for the Prism platform.
//!
//! This crate provides:
//! - Compile-time OCSF schema loading via `ocsf-proto-gen` and `prost-build`
//! - A `DescriptorPool` singleton for runtime protobuf reflection (`pool`)
//! - An `OcsfNormalizer` that converts raw sensor JSON to `DynamicMessage` (`normalizer`)
//! - An `EventClassSelector` mapping sensor+record_type to OCSF class_uid (`class_selector`)
//! - An `OcsfEnumMap` for integer-to-display-name lookups (`enum_map`)
//! - A compile-time OCSF version accessor (`version`)
//!
//! # Behavioral Contracts
//!
//! - BC-2.02.001: OCSF schema loaded at build time via ocsf-proto-gen
//! - BC-2.02.002: `OcsfNormalizer::normalize()` produces a valid `DynamicMessage`
//! - BC-2.02.009: OCSF schema version pinned at compile time, immutable at runtime
//! - BC-2.02.010: `OcsfEnumMap` returns display names for all enum values
//! - BC-2.02.012: `EventClassSelector` maps sensor+record_type to correct class_uid
//!
//! # Architecture Compliance
//!
//! Per `architecture/purity-boundary-map.md`, this crate is classified **pure-core**:
//! - `DescriptorPool` initialization uses `OnceLock` — never `Mutex<Option<...>>`
//! - `normalize()` must not panic — all errors returned via `Result`
//! - `EventClassSelector` is a compile-time constant — no runtime config of mappings
//! - `OcsfNormalizer` is `Send + Sync` (used from async tokio runtime)

pub mod class_selector;
pub mod enum_map;
pub mod normalizer;
pub mod pool;
pub mod version;

#[cfg(test)]
mod tests;

// Public re-exports (SS-02 API surface)
pub use class_selector::EventClassSelector;
pub use enum_map::OcsfEnumMap;
pub use normalizer::OcsfNormalizer;
pub use pool::OcsfDescriptors;
pub use version::ocsf_version;
