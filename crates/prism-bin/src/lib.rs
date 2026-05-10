//! prism-bin — Binary chassis, CLI, boot sequence, and signal handlers.
//!
//! This crate is the **only** `[[bin]]` target in the Prism workspace (ADR-022 §A).
//! It is Layer 4 in the workspace layered architecture; all library crates are
//! layers 0–3 and MUST NOT depend on `prism-bin`.
//!
//! # Modules
//!
//! - [`cli`] — clap 4.x CLI definition: 4 subcommands + exit-code documentation
//! - [`boot`] — 11-step boot orchestrator (BC-2.22.001; ADR-022 §B)
//! - [`exit_codes`] — canonical exit-code constants (ADR-022 §A)
//! - [`signals`] — SIGTERM + SIGHUP tokio signal handlers (BC-2.10.010)
//!
//! # Architecture Compliance
//!
//! - `prism-bin` MUST NOT be a dependency of any library crate (circular
//!   dependency forbidden; build fails if any library crate gains this dep).
//! - Steps 1–6 production paths MUST contain no `todo!()` before
//!   `S-WAVE5-PREP-01` transitions to `merged` (POL-12).
//!
//! # Subsystem
//!
//! SS-22 (Process Lifecycle) per `specs/architecture/module-decomposition.md`.

// Re-exported modules — all public for integration test access from boot_tests.rs.
pub mod boot;
pub mod cli;
pub mod exit_codes;
pub mod signals;

// Top-level re-exports for ergonomic import in integration tests.
pub use boot::{BootContext, BootError, PrismConfig, RunningServer};
pub use cli::{CliArgs, LogFormat, PrismCommand};
pub use exit_codes::{
    EXIT_CONFIG_INVALID, EXIT_GENERIC_ERROR, EXIT_INTERNAL_ERROR, EXIT_PERMISSION_DENIED,
    EXIT_SENSOR_FAIL, EXIT_SUCCESS,
};
