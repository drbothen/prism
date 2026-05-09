//! CLI argument parsing for the `prism` binary.
//!
//! Uses clap 4.x `#[derive(Parser)]` to define all subcommands per ADR-022 §A.
//! This module is pure (arg parsing only; no I/O until subcommand dispatch).
//!
//! # Exit-Code Contract (ADR-022 §A canonical table)
//!
//! ```text
//! 0  — success / clean shutdown
//! 1  — unhandled error (generic; includes unexpected panics caught by panic hook)
//! 2  — config-invalid (TOML parse error, schema validation failure, credential ref failure)
//! 3  — sensor-fail (a required sensor adapter failed to initialize at boot)
//! 4  — internal-error (runtime invariant violation; RocksDB open failed; audit init failed)
//! 5  — permission-denied (credential store access denied at boot)
//! ```

use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// Prism MCP server — sensor management platform for security analysts.
///
/// Exit codes:
///   0  success / clean shutdown
///   1  unhandled error (caught panic)
///   2  config-invalid (TOML parse error, schema validation, credential ref failure)
///   3  sensor-fail (required sensor adapter failed to initialize)
///   4  internal-error (RocksDB, QueryEngine, audit init failure)
///   5  permission-denied (credential store access denied)
#[derive(Debug, Parser)]
#[command(name = "prism", about = "Prism MCP server for MSSP sensor management")]
#[command(version)]
pub struct CliArgs {
    /// Override the config directory (default: ~/.prism/).
    /// Env var: PRISM_CONFIG_DIR.
    #[arg(long, global = true, env = "PRISM_CONFIG_DIR")]
    pub config_dir: Option<PathBuf>,

    /// Log format: "json" or "pretty" (default: pretty).
    /// Env var: PRISM_LOG_FORMAT.
    #[arg(
        long,
        global = true,
        env = "PRISM_LOG_FORMAT",
        default_value = "pretty"
    )]
    pub log_format: LogFormat,

    #[command(subcommand)]
    pub command: PrismCommand,
}

/// Log output format.
#[derive(Debug, Clone, clap::ValueEnum)]
pub enum LogFormat {
    /// Structured JSON lines (machine-readable; use for production deployments).
    Json,
    /// Human-readable pretty output (default for development).
    Pretty,
}

/// Prism subcommands (minimum viable set per ADR-022 §A).
///
/// Deferred subcommands (post-MVP): `migrate`, `debug-sensor`, `shell`.
/// These are NOT included per ADR-022 §A: "MUST NOT block S-WAVE5-PREP-01".
#[derive(Debug, Subcommand)]
pub enum PrismCommand {
    /// Boot and serve (blocks until SIGTERM/Ctrl-C).
    ///
    /// Exit codes: 0 clean, 1 panic, 2 config-invalid, 3 sensor-fail,
    ///   4 internal-error, 5 permission-denied.
    Start,

    /// Execute a single PrismQL query and write JSON result to stdout.
    ///
    /// Exit codes: 0 result, 1 panic, 2 config-invalid, 3 sensor-fail.
    Query {
        /// The PrismQL query string to execute.
        #[arg(value_name = "QUERY")]
        query_str: String,
    },

    /// Parse config and sensor TOMLs; report validity; exit 0 or 2.
    ///
    /// Exit codes: 0 valid, 2 config-invalid.
    ValidateConfig,

    /// Print semantic version and build metadata; exit 0.
    ///
    /// Exit codes: 0 always.
    Version,
}
