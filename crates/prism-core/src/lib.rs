//! prism-core — foundational types for the Prism platform.
//!
//! S-1.08 worktree stub: includes capability and error modules needed by
//! prism-security. Mirrors types from S-1.01 (error taxonomy) and S-1.03
//! (capability resolution) with `unimplemented!()` bodies (Red Gate).

// cfg(kani) is set by the Kani verification toolchain, not by Cargo features.
#![allow(unexpected_cfgs)]

pub mod capability;
pub mod error;

// Kani proofs — compiled only under `cargo kani` (cfg(kani) gate).
#[cfg(kani)]
pub mod proofs;

// ─────────────────────────────────────────────────────────────
// Public re-exports
// ─────────────────────────────────────────────────────────────

pub use capability::{
    CapabilityEffect, CapabilityExplanation, CapabilityPath, ClientCapabilities,
};
pub use error::PrismError;
