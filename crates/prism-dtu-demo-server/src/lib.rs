//! `prism-dtu-demo-server` — Unified multi-clone demo harness for Prism DTU clones.
//!
//! This crate is gated behind `#[cfg(any(test, feature = "dtu"))]` and must NEVER
//! link into a production binary. The binary target enforces this via
//! `required-features = ["dtu"]` in `Cargo.toml`.
//!
//! # Architecture
//!
//! The demo harness boots multiple DTU clones in a single process. It orchestrates
//! the six merged DTU clones via their `BehavioralClone` trait interfaces
//! (`prism-dtu-crowdstrike`, `-claroty`, `-cyberint`, `-armis`, `-threatintel`, `-nvd`).
//! It does NOT duplicate any clone logic.
//!
//! See `S-6.20` story spec and `ADR-002 Amendment` for the design rationale.
#![cfg(any(test, feature = "dtu"))]

pub mod config;
pub mod harness;
pub mod tls;

// Re-export primary types for test usage.
pub use config::DemoConfig;
pub use harness::{ClonePair, DemoHarness, StartReport};
