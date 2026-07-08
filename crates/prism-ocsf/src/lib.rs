//! `prism-ocsf` — OCSF normalization infrastructure for the Prism platform.
//!
//! S-1.05 extends S-1.04 with:
//! - `SensorMapper` trait and `SpecDrivenMapper` config-driven implementation
//! - `AliasResolver` for four-tier field resolution (alias module)
//! - `OcsfEvent` wrapper type
//! - Updated `OcsfNormalizer` accepting `Vec<Box<dyn SensorMapper>>`
//!
//! # Behavioral Contracts
//!
//! - BC-2.02.003: CrowdStrike field mapping (via SpecDrivenMapper + TOML spec)
//! - BC-2.02.004: Cyberint field mapping (via SpecDrivenMapper + TOML spec)
//! - BC-2.02.005: Claroty xDome field mapping (via SpecDrivenMapper + TOML spec)
//! - BC-2.02.006: Armis Centrix field mapping (via SpecDrivenMapper + TOML spec)
//! - BC-2.02.007: Unmapped fields preserved in raw_extensions (VP-017)
//! - BC-2.02.008: Four-tier field alias resolution
//! - BC-2.02.011: Normalization errors include source record ID + specific reason

pub mod alias;
pub mod class_selector;
pub mod enum_map;
pub mod event;
pub mod mappers;
pub mod normalizer;
pub mod pool;
pub mod version;

#[cfg(test)]
mod tests;

// Public re-exports (SS-02 API surface)
pub use alias::{AliasResolver, AliasResult};
pub use class_selector::EventClassSelector;
pub use enum_map::OcsfEnumMap;
pub use event::OcsfEvent;
pub use mappers::{SensorMapper, SpecDrivenMapper};
pub use normalizer::{shared_enum_map, OcsfNormalizer, OCSF_ENUM_LABEL_FIELDS};
pub use pool::OcsfDescriptors;
pub use version::ocsf_version;
