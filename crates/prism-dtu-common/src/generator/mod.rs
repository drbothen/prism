//! Generator shared types — `Archetype`, `GenOpts`, `FixtureSet`, `Provenance`,
//! `OrgId`, `seeded_rng`, `default_page_size`, `apply_overrides`.
//!
//! All symbols in this module are gated behind `#[cfg(feature = "fixture-gen")]`
//! at the crate level (AC-007 / D-056).

pub mod archetype;
pub mod fixture;
pub mod offset;
pub mod opts;
pub mod pagination;
pub mod rng;

pub use archetype::{all_archetypes, Archetype};
pub use fixture::{apply_overrides, FixtureSet, OrgId, Provenance};
pub use offset::stable_offset;
pub use opts::{demo_time_anchor, GenOpts, GenOptsError, DEMO_TIME_ANCHOR_EPOCH_SECS};
pub use pagination::default_page_size;
pub use rng::seeded_rng;
