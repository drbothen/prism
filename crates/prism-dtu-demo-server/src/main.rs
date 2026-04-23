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

/// Name of the PID sidecar file written in cwd by `start`.
const PID_FILE: &str = ".prism-dtu-demo-server.pid";

/// Name of the URL map sidecar file written in cwd by `start`.
const URL_FILE: &str = ".prism-dtu-demo-server.urls.json";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start {
            config,
            tls,
            bind_any,
            deterministic_logging,
        } => cmd_start(config, tls, bind_any, deterministic_logging).await,
        Commands::Stop => cmd_stop(),
        Commands::Configure { clone, json } => cmd_configure(clone, json).await,
    }
}

// ---------------------------------------------------------------------------
// `start` subcommand
// ---------------------------------------------------------------------------

async fn cmd_start(
    config_path: std::path::PathBuf,
    tls: bool,
    bind_any: bool,
    deterministic_logging: bool,
) -> anyhow::Result<()> {
    // 1. Initialise tracing.
    init_tracing(deterministic_logging);

    // 2. Load TOML config.
    let config = prism_dtu_demo_server::config::DemoConfig::from_file(&config_path)
        .map_err(|e| anyhow::anyhow!("Failed to load config {:?}: {}", config_path, e))?;

    // 3. Enforce R-DEMO-001 two-factor gate for --bind-any.
    if bind_any {
        let allow_env = std::env::var("PRISM_DTU_DEMO_ALLOW_NETWORK_BIND").unwrap_or_default();
        if allow_env != "I-UNDERSTAND-THE-RISK" {
            anyhow::bail!(
                "--bind-any was set but PRISM_DTU_DEMO_ALLOW_NETWORK_BIND is not \
                 'I-UNDERSTAND-THE-RISK'. Both are required for non-loopback binding \
                 (R-DEMO-001). Export the env var to proceed."
            );
        }
    }

    // 4. TLS: generate self-signed cert if requested.
    handle_tls(tls)?;

    // 5. Build clone pairs and harness.
    let pairs = prism_dtu_demo_server::harness::build_clone_pairs(&config)
        .map_err(|e| anyhow::anyhow!("Failed to build clone pairs: {}", e))?;
    let mut harness = prism_dtu_demo_server::DemoHarness::new(pairs);

    // 6. Start all clones.
    harness
        .start_all(&config)
        .await
        .map_err(|e| anyhow::anyhow!("Harness startup failed: {}", e))?;

    // 7. Write PID file (atomic tmp + rename).
    write_pid_file()?;

    // 8. Write URL sidecar for `configure` subcommand.
    write_url_sidecar(&harness)?;

    // 9. Print URL table.
    harness.print_url_table();

    // 10. Print StartReport as JSON to stdout (one line).
    let report = harness.last_start_report();
    let report_json = serialize_start_report(report);
    println!(
        "{}",
        serde_json::to_string(&report_json).unwrap_or_else(|_| "{}".to_string())
    );

    // 11. Install SIGINT/SIGTERM handler and keep process alive.
    wait_for_shutdown_signal(&mut harness).await;

    Ok(())
}

/// A serializable view of `StartReport` (since `std::io::Error` is not `Serialize`).
#[derive(serde::Serialize)]
struct StartReportJson {
    successfully_started: Vec<String>,
    cleaned_up_after_failure: Vec<String>,
    failed_at: Option<FailedAtJson>,
    skipped_due_to_error: Vec<SkippedJson>,
}

#[derive(serde::Serialize)]
struct FailedAtJson {
    name: String,
    error: String,
}

#[derive(serde::Serialize)]
struct SkippedJson {
    name: String,
    error: String,
}

fn serialize_start_report(report: &prism_dtu_demo_server::StartReport) -> StartReportJson {
    StartReportJson {
        successfully_started: report.successfully_started.clone(),
        cleaned_up_after_failure: report.cleaned_up_after_failure.clone(),
        failed_at: report.failed_at.as_ref().map(|(name, err)| FailedAtJson {
            name: name.clone(),
            error: err.to_string(),
        }),
        skipped_due_to_error: report
            .skipped_due_to_error
            .iter()
            .map(|(name, err)| SkippedJson {
                name: name.clone(),
                error: err.to_string(),
            })
            .collect(),
    }
}

/// Initialise `tracing-subscriber`.
///
/// When `deterministic_logging` is true: use a compact format without timestamps
/// or PIDs, making log output reproducible for the same request sequence (AC-7).
fn init_tracing(deterministic_logging: bool) {
    use tracing_subscriber::fmt;

    if deterministic_logging {
        // No timestamps, no ANSI, no PIDs — fully deterministic output.
        fmt()
            .without_time()
            .with_ansi(false)
            .with_target(false)
            .with_thread_ids(false)
            .with_thread_names(false)
            .init();
    } else {
        fmt().with_target(true).init();
    }
}

/// Handle TLS flag: feature-gated, generate and print cert fingerprint.
#[allow(unused_variables)]
fn handle_tls(tls: bool) -> anyhow::Result<()> {
    if !tls {
        return Ok(());
    }

    #[cfg(feature = "tls")]
    {
        let (cert_pem, _key_pem) = prism_dtu_demo_server::tls::inner::generate_self_signed_cert()?;
        prism_dtu_demo_server::tls::inner::print_cert_fingerprint(&cert_pem);
        Ok(())
    }

    #[cfg(not(feature = "tls"))]
    {
        anyhow::bail!(
            "--tls was requested but this binary was not compiled with the `tls` feature. \
             Rebuild with `--features tls` to enable TLS support."
        );
    }
}

/// Write PID file atomically (tmp + rename).
fn write_pid_file() -> anyhow::Result<()> {
    let pid = std::process::id();
    let tmp_path = format!("{PID_FILE}.tmp");
    std::fs::write(&tmp_path, pid.to_string())
        .map_err(|e| anyhow::anyhow!("Failed to write PID tmp file: {}", e))?;
    std::fs::rename(&tmp_path, PID_FILE)
        .map_err(|e| anyhow::anyhow!("Failed to rename PID file: {}", e))?;
    Ok(())
}

/// Write the URL sidecar JSON file so that `configure` can look up clone URLs.
fn write_url_sidecar(harness: &prism_dtu_demo_server::DemoHarness) -> anyhow::Result<()> {
    let url_map = harness.url_map();
    let json = serde_json::to_string(&url_map)
        .map_err(|e| anyhow::anyhow!("Failed to serialise URL map: {}", e))?;
    let tmp_path = format!("{URL_FILE}.tmp");
    std::fs::write(&tmp_path, &json)
        .map_err(|e| anyhow::anyhow!("Failed to write URL sidecar tmp: {}", e))?;
    std::fs::rename(&tmp_path, URL_FILE)
        .map_err(|e| anyhow::anyhow!("Failed to rename URL sidecar: {}", e))?;
    Ok(())
}

/// Wait for SIGINT or SIGTERM, then gracefully shut down all clones.
///
/// If shutdown takes longer than 5 seconds, exits with code 1.
async fn wait_for_shutdown_signal(harness: &mut prism_dtu_demo_server::DemoHarness) {
    // Await either Ctrl-C (SIGINT) or SIGTERM.
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};

        let mut sigterm =
            signal(SignalKind::terminate()).expect("failed to install SIGTERM handler");

        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                tracing::info!("Received SIGINT — initiating graceful shutdown");
            }
            _ = sigterm.recv() => {
                tracing::info!("Received SIGTERM — initiating graceful shutdown");
            }
        }
    }

    #[cfg(not(unix))]
    {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl-C handler");
        tracing::info!("Received Ctrl-C — initiating graceful shutdown");
    }

    // Graceful shutdown with 5-second timeout.
    let stop_result =
        tokio::time::timeout(std::time::Duration::from_secs(5), harness.stop_all()).await;

    // Remove sidecar files.
    let _ = std::fs::remove_file(PID_FILE);
    let _ = std::fs::remove_file(URL_FILE);

    if stop_result.is_err() {
        tracing::error!("stop_all() timed out after 5s — hard aborting");
        std::process::exit(1);
    }

    tracing::info!("Harness stopped cleanly.");
}

// ---------------------------------------------------------------------------
// `stop` subcommand
// ---------------------------------------------------------------------------

fn cmd_stop() -> anyhow::Result<()> {
    let pid_str = std::fs::read_to_string(PID_FILE).map_err(|_| {
        anyhow::anyhow!(
            "PID file '{}' not found — is the harness running?",
            PID_FILE
        )
    })?;

    let pid: i32 = pid_str.trim().parse().map_err(|_| {
        anyhow::anyhow!(
            "PID file '{}' contains invalid PID: {:?}",
            PID_FILE,
            pid_str.trim()
        )
    })?;

    send_sigterm(pid)?;
    println!("sent SIGTERM to pid {pid}");
    Ok(())
}

/// Send SIGTERM to `pid`.
#[cfg(unix)]
fn send_sigterm(pid: i32) -> anyhow::Result<()> {
    // SAFETY: kill(2) is safe to call with a valid PID and signal number.
    let ret = unsafe { libc::kill(pid, libc::SIGTERM) };
    if ret != 0 {
        let err = std::io::Error::last_os_error();
        anyhow::bail!("kill({pid}, SIGTERM) failed: {err}");
    }
    Ok(())
}

#[cfg(not(unix))]
fn send_sigterm(pid: i32) -> anyhow::Result<()> {
    anyhow::bail!("stop subcommand is only supported on Unix platforms (pid={pid})");
}

// ---------------------------------------------------------------------------
// `configure` subcommand
// ---------------------------------------------------------------------------

async fn cmd_configure(clone_name: String, json_body: String) -> anyhow::Result<()> {
    // Read the URL sidecar written by `start`.
    let sidecar_str = std::fs::read_to_string(URL_FILE).map_err(|_| {
        anyhow::anyhow!(
            "URL sidecar '{}' not found — is the harness running?",
            URL_FILE
        )
    })?;

    let url_map: std::collections::HashMap<String, String> = serde_json::from_str(&sidecar_str)
        .map_err(|e| anyhow::anyhow!("Failed to parse URL sidecar: {}", e))?;

    let clone_url = url_map.get(&clone_name).ok_or_else(|| {
        anyhow::anyhow!(
            "Clone '{}' not found in running harness. Available: {:?}",
            clone_name,
            url_map.keys().collect::<Vec<_>>()
        )
    })?;

    let configure_url = format!("{clone_url}/dtu/configure");

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to build HTTP client: {}", e))?;

    let resp = client
        .post(&configure_url)
        .header("Content-Type", "application/json")
        .body(json_body)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("POST to {} failed: {}", configure_url, e))?;

    let status = resp.status();
    let body = resp
        .text()
        .await
        .unwrap_or_else(|_| "<failed to read body>".to_string());

    println!("HTTP {status}");
    println!("{body}");

    if !status.is_success() {
        std::process::exit(1);
    }

    Ok(())
}
