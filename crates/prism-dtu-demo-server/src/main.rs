//! `prism-dtu-demo-server` binary entry point.
//!
//! Boots multiple DTU clones in a single process for live demos and CI regression.
//!
//! # Feature gate
//!
//! Feature gating is handled entirely by `required-features = ["dtu"]` in `Cargo.toml`.
//! This file does NOT contain `#![cfg(feature = "dtu")]` — cargo skips the binary target
//! entirely when the `dtu` feature is absent (AC-8).
//!
//! # Usage
//!
//! ```sh
//! cargo run -p prism-dtu-demo-server --features dtu -- start --config configs/demo.toml
//! ```
//!
//! # Security Warning (R-DEMO-001)
//!
//! Non-loopback binding requires BOTH `--bind-any` AND the environment variable
//! `PRISM_DTU_DEMO_ALLOW_NETWORK_BIND=I-UNDERSTAND-THE-RISK`.

use clap::{Parser, Subcommand};

/// Unified multi-clone demo harness for Prism DTU clones.
#[derive(Debug, Parser)]
#[command(name = "prism-dtu-demo-server", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Start the demo harness with the given config file.
    Start {
        /// Path to the demo config TOML file (e.g. `configs/demo.toml`).
        #[arg(long, short = 'c', value_name = "PATH")]
        config: std::path::PathBuf,

        /// Enable TLS (requires the `tls` feature to be compiled in).
        #[arg(long)]
        tls: bool,

        /// Allow binding to non-loopback addresses.
        ///
        /// Also requires `PRISM_DTU_DEMO_ALLOW_NETWORK_BIND=I-UNDERSTAND-THE-RISK` (R-DEMO-001).
        #[arg(long)]
        bind_any: bool,

        /// Suppress timestamps, PIDs, and request-ids from log output.
        ///
        /// Combined with `seed = 42` in `demo.toml`, makes log output reproducible
        /// across runs for the same request sequence (AC-7).
        #[arg(long)]
        deterministic_logging: bool,
    },

    /// Send SIGTERM to a backgrounded harness PID (reads `.prism-dtu-demo-server.pid`).
    Stop,

    /// Convenience wrapper: POST to a clone's own `/dtu/configure` endpoint.
    Configure {
        /// Clone name (e.g. `crowdstrike`, `cyberint`).
        clone: String,
        /// JSON payload to send.
        json: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start {
            config,
            tls,
            bind_any,
            deterministic_logging,
        } => {
            todo!(
                "start subcommand not yet implemented — implement in S-6.20 Phase 2 \
                 (config={:?}, tls={}, bind_any={}, deterministic_logging={})",
                config,
                tls,
                bind_any,
                deterministic_logging
            )
        }
        Commands::Stop => {
            todo!("stop subcommand not yet implemented — implement in S-6.20 Phase 2")
        }
        Commands::Configure { clone, json } => {
            todo!(
                "configure subcommand not yet implemented — implement in S-6.20 Phase 2 \
                 (clone={:?}, json={:?})",
                clone,
                json
            )
        }
    }
}
