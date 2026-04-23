//! `prism-ocsf` — OCSF normalization infrastructure for the Prism platform.
//!
//! S-1.05 extends S-1.04 with:
//! - `SensorMapper` trait and four sensor implementations (mappers module)
//! - `AliasResolver` for four-tier field resolution (alias module)
//! - `OcsfEvent` wrapper type
//! - Updated `OcsfNormalizer` accepting `Vec<Box<dyn SensorMapper>>`
//!
//! # Behavioral Contracts
//!
//! - BC-2.02.003: CrowdStrike field mapping
//! - BC-2.02.004: Cyberint field mapping (multi-format timestamps)
//! - BC-2.02.005: Claroty xDome field mapping (9 data sources, polymorphic IDs)
//! - BC-2.02.006: Armis Centrix field mapping (7 data sources, AQL forwarding)
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
pub use mappers::{ArmisMapper, ClarotyMapper, CrowdStrikeMapper, CyberintMapper, SensorMapper};
pub use normalizer::OcsfNormalizer;
pub use pool::OcsfDescriptors;
pub use version::ocsf_version;
