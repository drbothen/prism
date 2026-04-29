//! Generator shared types — `Archetype`, `GenOpts`, `FixtureSet`, `Provenance`,
//! `OrgId`, `seeded_rng`, `default_page_size`, `apply_overrides`.
//!
//! All symbols in this module are gated behind `#[cfg(feature = "fixture-gen")]`
//! at the crate level (AC-007 / D-056).

pub mod archetype;
pub mod fixture;
pub mod opts;
pub mod pagination;
pub mod rng;

pub use archetype::{all_archetypes, Archetype};
pub use fixture::{apply_overrides, FixtureSet, OrgId, Provenance};
pub use opts::{GenOpts, GenOptsError};
pub use pagination::default_page_size;
pub use rng::seeded_rng;
