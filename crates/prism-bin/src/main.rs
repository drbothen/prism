//! Prism binary entry point.
//!
//! Responsibilities (ADR-022 §A):
//! 1. Register custom panic hook (before any other code) — AC-12.
//! 2. Parse CLI args via clap.
//! 3. Dispatch to the appropriate subcommand handler.
//! 4. Map top-level errors to canonical exit codes (ADR-022 §A exit-code contract).
//!
//! # Panic Hook
//!
//! The panic hook is installed BEFORE `tracing_subscriber` is initialized.
//! If tracing is not yet active when a panic fires, the hook falls back to
//! `eprintln!` for the log line (BC-2.10.010; AC-12 requirement).
//!
//! # Exit-Code Contract (ADR-022 §A canonical table)
//!
//! See `exit_codes.rs` for the canonical constants.

use std::process;

use clap::Parser;

use prism_bin::cli::CliArgs;
#[allow(unused_imports)] // Red Gate scaffold: exit codes used by dispatch() once implemented
use prism_bin::exit_codes::{EXIT_CONFIG_INVALID, EXIT_GENERIC_ERROR, EXIT_INTERNAL_ERROR};

/// Multi-thread tokio runtime per AD-013.
#[tokio::main(flavor = "multi_thread")]
async fn main() {
    // Step 0: Register custom panic hook FIRST — before any other code.
    // This ensures panics are always observable as structured log entries (AC-12).
    // The hook calls process::exit(1) after emitting the error log.
    // If tracing is not initialized yet, falls back to eprintln! (ADR-022 §A).
    install_panic_hook();

    // Parse CLI args.
    let args = CliArgs::parse();

    // Dispatch subcommand.
    let exit_code = dispatch(args).await;
    process::exit(exit_code);
}

/// Dispatch to the appropriate subcommand handler.
///
/// Returns the canonical exit code for the subcommand outcome.
async fn dispatch(_args: CliArgs) -> i32 {
    todo!("S-WAVE5-PREP-01: dispatch subcommand; initialize tracing (step 1) first; call boot sequence for start/validate-config; return canonical exit code per ADR-022 §A")
}

/// Install the custom panic hook (ADR-022 §A; AC-12).
///
/// The hook emits a `tracing::error!` log before calling `process::exit(1)`.
/// If tracing is not yet initialized, falls back to `eprintln!` as a safety
/// net so the panic is always observable in some channel.
///
/// MUST be called before `tracing_subscriber::init()` to avoid a race.
fn install_panic_hook() {
    todo!("S-WAVE5-PREP-01: install std::panic::set_hook that emits tracing::error! then calls process::exit(1); fall back to eprintln! if tracing not yet initialized; AC-12")
}
