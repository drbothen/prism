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

use prism_bin::boot::{self, PrismConfig};
use prism_bin::cli::{CliArgs, LogFormat, PrismCommand};
use prism_bin::exit_codes::{EXIT_CONFIG_INVALID, EXIT_INTERNAL_ERROR, EXIT_SUCCESS};

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
async fn dispatch(args: CliArgs) -> i32 {
    // Short-circuit: version subcommand never needs config resolution or tracing.
    // Defense-in-depth: `prism version` must exit 0 even when HOME/APPDATA is unset.
    // This check runs BEFORE step1_init_tracing and BEFORE resolve_config_dir.
    if let PrismCommand::Version = &args.command {
        // AC-2: print "prism X.Y.Z" to stdout; exit 0.
        println!("prism {}", env!("CARGO_PKG_VERSION"));
        return EXIT_SUCCESS;
    }

    // Initialize tracing (step 1) first before any other processing.
    boot::step1_init_tracing(&args.log_format);

    // Resolve config directory from CLI arg or PRISM_CONFIG_DIR env var.
    // The --config-dir / PRISM_CONFIG_DIR resolution is already done by clap
    // (the field is annotated with env = "PRISM_CONFIG_DIR"), so args.config_dir
    // holds the resolved value. For the platform default, dirs::config_dir()
    // returns the platform-appropriate config directory:
    //   - Linux:   $XDG_CONFIG_HOME/prism (if XDG_CONFIG_HOME is set and absolute) else ~/.config/prism/
    //   - macOS:   ~/Library/Application Support/prism/
    //   - Windows: %APPDATA%\prism\  (e.g., C:\Users\<user>\AppData\Roaming\prism\)
    let config_dir = match args.config_dir {
        Some(d) => d,
        None => match dirs::config_dir().map(|d| d.join("prism")) {
            Some(dir) => dir,
            None => {
                eprintln!(
                    "Could not determine config directory for default path. \
                         Set PRISM_CONFIG_DIR explicitly."
                );
                process::exit(EXIT_CONFIG_INVALID);
            }
        },
    };

    match args.command {
        PrismCommand::Version => {
            // Unreachable: handled by the short-circuit at the top of dispatch().
            // Kept for exhaustive match completeness (compiler requires all arms).
            // AC-2: print "prism X.Y.Z" to stdout; exit 0.
            println!("prism {}", env!("CARGO_PKG_VERSION"));
            EXIT_SUCCESS
        }

        PrismCommand::ValidateConfig => {
            // Run boot steps 1-6; if they all complete, config is valid → exit 0.
            // Tracing already initialized in step 1 above.
            match boot::boot_to_step_6(&config_dir).await {
                Ok(_ctx) => {
                    tracing::info!("Config validation passed — all boot steps 1-6 completed");
                    EXIT_SUCCESS
                }
                Err(e) => {
                    let code = e.exit_code();
                    eprintln!("prism validate-config failed: {e}");
                    code
                }
            }
        }

        PrismCommand::Start => {
            // Run the canonical full boot sequence (steps 1-11).
            //
            // `run_boot_sequence` intercalates step 7.5 (plugin-load) between step 7
            // (storage init) and step 8 (query-engine init) per BC-2.22.001 §Sequencing
            // Invariant and ADR-023 §C4 pre-traffic gate (POL-15 enforcement).
            //
            // Steps 7-11 are todo!() stubs for sibling stories (S-3.02-FOLLOWUP-RUNTIME,
            // S-5.01-FOLLOWUP-MCP-BOOT, S-1.12-FOLLOWUP). The process will panic at the
            // first todo!() after step 7.5 — caught by the panic hook → exit 1.
            // Step 7.5 (plugin-load) WILL execute before the first todo!() because
            // run_boot_sequence positions it at step 7.5 within the sequence.
            match boot::run_boot_sequence(&config_dir).await {
                Ok(_server) => EXIT_SUCCESS,
                Err(e) => {
                    let code = e.exit_code();
                    eprintln!("prism start failed: {e}");
                    code
                }
            }
        }

        PrismCommand::Query { query_str: _ } => {
            // QueryEngine::execute is todo!() until S-3.02-FOLLOWUP-RUNTIME.
            // AC-11: must not return exit 2 (that's for unknown subcommand).
            // Return exit 4 (internal-error) because QueryEngine is not yet initialized.
            eprintln!(
                "prism query: QueryEngine not yet implemented \
                 (deferred to S-3.02-FOLLOWUP-RUNTIME); exit 4"
            );
            EXIT_INTERNAL_ERROR
        }
    }
}

/// Install the custom panic hook (ADR-022 §A; AC-12).
///
/// The hook emits a `tracing::error!` log before calling `process::exit(1)`.
/// If tracing is not yet initialized, falls back to `eprintln!` as a safety
/// net so the panic is always observable in some channel.
///
/// MUST be called before `tracing_subscriber::init()` to avoid a race.
fn install_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        // Attempt structured log first.
        // If tracing is not initialized, this is a no-op (the subscriber is not set yet).
        // We use try/catch pattern via the tracing macros which silently drop if no subscriber.
        let location = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown location".to_string());

        let payload = if let Some(s) = info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "non-string panic payload".to_string()
        };

        // Emit via tracing (no-op if subscriber not yet initialized).
        tracing::error!(
            panic.payload = %payload,
            panic.location = %location,
            "Prism process panicked — exiting with code 1 (AC-12; ADR-022 §A panic hook)"
        );

        // Always emit to stderr as fallback (in case tracing not initialized).
        eprintln!(
            "PANIC at {location}: {payload}\n\
             Prism process panicked — exiting with code 1 (AC-12)"
        );

        // AC-12: exit code 1 (not 101 which is Rust's default).
        process::exit(1);
    }));
}
