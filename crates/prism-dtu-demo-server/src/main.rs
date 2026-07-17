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

    /// Start all orgs' clone fleets using the multi-instance API.
    ///
    /// Requires `--features dtu,fixture-gen` — the seeded clone constructors
    /// (`new_with_seed`) are `#[cfg(feature = "fixture-gen")]`-gated. Omitting
    /// `fixture-gen` causes a hard error (compile_error! or runtime panic) to
    /// prevent silent fallback to unseeded `new()` which would violate
    /// INV-DISTINCT-DATA-001 (org-a and org-c would serve identical data).
    StartMulti {
        /// Path to the multi-org demo config TOML (e.g. `scripts/demo.toml`).
        #[arg(long, short = 'c', value_name = "PATH")]
        config: std::path::PathBuf,
    },

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

// URL_FILE, URL_MULTI_FILE, TOKEN_FILE, TOKEN_MULTI_FILE are defined in lib.rs (as `pub const`)
// so that multi_org_cmd.rs can reference them in error messages. Import them here.
use prism_dtu_demo_server::{TOKEN_FILE, TOKEN_MULTI_FILE, URL_FILE, URL_MULTI_FILE};

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
        Commands::StartMulti { config } => cmd_start_multi(config).await,
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
    //    Returns Some(RustlsConfig) when tls=true and the tls feature is enabled.
    //    Returns None when tls=false.
    //    Errors (returns Err) when tls=true but the tls feature is absent.
    //
    //    STDOUT ORDERING (per TD-WV1-04 AC-7):
    //      1. sha256: fingerprint line  (printed INSIDE handle_tls before returning)
    //      2. URL table                 (printed below in step 9)
    //      3. StartReport JSON          (printed below in step 10)

    // Install rustls crypto provider (required before any rustls TLS operations).
    // This is a no-op if already installed; safe to call unconditionally.
    #[cfg(feature = "tls")]
    {
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    }

    #[cfg(feature = "tls")]
    let tls_config: Option<std::sync::Arc<axum_server::tls_rustls::RustlsConfig>> =
        handle_tls(tls).await?;

    #[cfg(not(feature = "tls"))]
    let tls_config: Option<()> = handle_tls(tls)?;

    // 5. Build clone pairs and harness.
    let pairs = prism_dtu_demo_server::harness::build_clone_pairs(&config)
        .map_err(|e| anyhow::anyhow!("Failed to build clone pairs: {}", e))?;
    let mut harness = prism_dtu_demo_server::DemoHarness::new(pairs);

    // 6. Start all clones (TLS config propagated to each clone's start_on).
    harness
        .start_all(&config, tls_config)
        .await
        .map_err(|e| anyhow::anyhow!("Harness startup failed: {}", e))?;

    // 7. Write PID file (atomic tmp + rename).
    write_pid_file()?;

    // 8. Write URL sidecar for `configure` subcommand.
    write_url_sidecar(&harness)?;

    // 8b. Write admin-token sidecar for `configure` subcommand (T-04).
    //     Written atomically alongside URL_FILE so that `cmd_configure` (a separate
    //     process invocation) can obtain per-clone tokens for the `X-Admin-Token` header
    //     required by ADR-003 Amendment #5 (DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001 T-04).
    write_token_sidecar(&harness)?;

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

/// Handle TLS flag: feature-gated, generate cert, print fingerprint.
///
/// Returns `Ok(Some(Arc<RustlsConfig>))` when `tls=true` and the `tls` feature is enabled.
/// The fingerprint is printed to stdout BEFORE returning (AC-7 ordering: fingerprint first).
/// Returns `Ok(None)` when `tls=false`.
/// Returns `Err` when `tls=true` but the `tls` feature is absent.
#[cfg(feature = "tls")]
async fn handle_tls(
    tls: bool,
) -> anyhow::Result<Option<std::sync::Arc<axum_server::tls_rustls::RustlsConfig>>> {
    if !tls {
        return Ok(None);
    }

    let (cert_pem, key_pem, cert_der) =
        prism_dtu_demo_server::tls::inner::generate_self_signed_cert()?;

    // Print fingerprint FIRST (AC-7 ordering: sha256: before URL table).
    prism_dtu_demo_server::tls::inner::print_cert_fingerprint(&cert_der);

    let rustls_cfg = prism_dtu_demo_server::tls::inner::build_rustls_config(&cert_pem, &key_pem)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to build RustlsConfig: {}", e))?;

    Ok(Some(std::sync::Arc::new(rustls_cfg)))
}

#[cfg(not(feature = "tls"))]
fn handle_tls(tls: bool) -> anyhow::Result<Option<()>> {
    if !tls {
        return Ok(None);
    }

    anyhow::bail!(
        "--tls was requested but this binary was not compiled with the `tls` feature. \
         Rebuild with `--features tls` to enable TLS support."
    );
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

/// Write the admin-token sidecar JSON file so that `configure` can look up per-clone
/// admin tokens for the `X-Admin-Token` header (ADR-003 Amendment #5).
///
/// Delegates to `write_token_sidecar_to_path` (the testable, path-parameterised variant
/// in `harness.rs`) with the canonical `TOKEN_FILE` path.
///
/// Written atomically (tmp + rename, 0600 on Unix) to prevent `cmd_configure` from reading
/// a partial file (GAP-3 sidecar-availability guarantee / F-ADMTOK-P1-OBS-002).
///
/// # T-03/T-04 (DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001)
///
/// Called immediately after `write_url_sidecar` in `cmd_start()` — the call site is
/// in this binary (main.rs), NOT in `DemoHarness::start_all`, so that the library does
/// not gain an unrequested I/O failure mode for callers that do not need the sidecar.
fn write_token_sidecar(harness: &prism_dtu_demo_server::DemoHarness) -> anyhow::Result<()> {
    prism_dtu_demo_server::write_token_sidecar_to_path(
        &harness.token_map(),
        std::path::Path::new(TOKEN_FILE),
    )
}

/// Wait for SIGINT or SIGTERM, then gracefully shut down all clones.
///
/// If shutdown takes longer than 5 seconds, exits with code 1.
async fn wait_for_shutdown_signal(harness: &mut prism_dtu_demo_server::DemoHarness) {
    // Await either Ctrl-C (SIGINT) or SIGTERM.
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};

        // SAFETY: failure to install a signal handler is a fatal setup error; panic is correct.
        #[allow(clippy::expect_used)]
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
        // SAFETY: failure to install a signal handler is a fatal setup error; panic is correct.
        #[allow(clippy::expect_used)]
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
    // T-09: Remove admin-token sidecar alongside URL sidecar on shutdown.
    let _ = std::fs::remove_file(TOKEN_FILE);

    if stop_result.is_err() {
        tracing::error!("stop_all() timed out after 5s — hard aborting");
        std::process::exit(1);
    }

    tracing::info!("Harness stopped cleanly.");
}

// ---------------------------------------------------------------------------
// `start-multi` subcommand — S-DEMO-LAUNCHER-CONSOLIDATION-001
// ---------------------------------------------------------------------------

/// Entry point for `prism-dtu-demo-server start-multi`.
///
/// Loads `MultiOrgDemoConfig` from `config_path`, starts all org clone fleets
/// via `start_multi_for_config`, writes the nested sidecar, then waits for shutdown.
///
/// # fixture-gen requirement
///
/// This function calls `build_multi_clone_factory` (via `start_multi_for_config`) which is
/// `#[cfg(feature = "fixture-gen")]`-only. Building without `fixture-gen` produces a runtime
/// panic — NEVER a silent fallback to unseeded `new()` (which would violate INV-DISTINCT-DATA-001).
async fn cmd_start_multi(config_path: std::path::PathBuf) -> anyhow::Result<()> {
    // 1. Initialise tracing (deterministic_logging=false for multi-org mode).
    init_tracing(false);

    // 2. Load MultiOrgDemoConfig.
    let cfg = prism_dtu_demo_server::config::MultiOrgDemoConfig::from_file(&config_path)
        .map_err(|e| anyhow::anyhow!("Failed to load multi-org config {:?}: {}", config_path, e))?;

    // 3. Start all org clone fleets. This calls build_multi_clone_factory (fixture-gen required)
    //    which will panic if fixture-gen is absent (GAP-1 enforcement).
    let servers = prism_dtu_demo_server::start_multi_for_config(&cfg)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to start multi-org clone fleet: {}", e))?;

    // 4. Write PID file (same atomic tmp+rename helper used by cmd_start).
    write_pid_file()?;

    // 5. Write NESTED URL sidecar: {org_slug: {sensor_id: url}}.
    //    Written to URL_MULTI_FILE (distinct from the flat URL_FILE written by `start`).
    write_multi_url_sidecar(&servers, &cfg)?;

    // 5b. Write NESTED admin-token sidecar: {org_slug: {sensor_id: token}}.
    //     Written to TOKEN_MULTI_FILE alongside URL_MULTI_FILE so that `cmd_configure`
    //     can obtain per-clone tokens for the `X-Admin-Token` header (T-06).
    write_multi_admin_token_sidecar(&servers, &cfg)?;

    // 6. Print nested URL table to stdout.
    let socket_map = servers.socket_map();
    println!("start-multi: {} instances running", socket_map.len());
    let mut entries: Vec<_> = socket_map.iter().collect();
    entries.sort_by_key(|(name, _)| name.as_str());
    for (name, addr) in &entries {
        println!("  {name}: http://{addr}");
    }

    // 7. Wait for SIGTERM/SIGINT, then gracefully shut down.
    wait_for_shutdown_signal_multi(&servers).await;

    Ok(())
}

/// Write the NESTED URL sidecar file `.prism-dtu-demo-server.urls-multi.json`.
///
/// Delegates to `write_multi_url_sidecar_to_path` (the testable, path-parameterised
/// variant in `multi_org_cmd.rs`) with the canonical `URL_MULTI_FILE` path.
///
/// The sidecar is written atomically (tmp + rename) to prevent demo-run.sh from
/// reading a partial file during the poll loop (GAP-3 sidecar-availability guarantee).
///
/// # Production-grade: no silent drops (MED-2 fix)
///
/// This function errors loudly if any expected `{org_slug}-{sensor_id}` entry is
/// absent from `servers.socket_map()`. The previous `filter_map` implementation
/// silently dropped missing entries — a production defect that would cause prism
/// boot failure for affected org×sensor queries (CLAUDE.md Standing Rule 3 §2).
fn write_multi_url_sidecar(
    servers: &prism_dtu_demo_server::MultiInstanceServers,
    cfg: &prism_dtu_demo_server::MultiOrgDemoConfig,
) -> anyhow::Result<()> {
    prism_dtu_demo_server::write_multi_url_sidecar_to_path(
        servers,
        cfg,
        std::path::Path::new(URL_MULTI_FILE),
    )
}

/// Write the NESTED admin-token sidecar file `.prism-dtu-demo-server.admin-tokens-multi.json`.
///
/// Delegates to `write_multi_admin_token_sidecar_to_path` (the testable, path-parameterised
/// variant in `multi_org_cmd.rs`) with the canonical `TOKEN_MULTI_FILE` path.
///
/// Written atomically (tmp + rename) to prevent `cmd_configure` from reading a partial file.
///
/// # Production-grade: no silent drops
///
/// Errors loudly if any expected `{org_slug}-{sensor_id}` entry is absent from
/// `servers.admin_token_map()` — a missing token would cause HTTP 401 on every
/// `configure` call for that sensor (CLAUDE.md Standing Rule 3 §2).
fn write_multi_admin_token_sidecar(
    servers: &prism_dtu_demo_server::MultiInstanceServers,
    cfg: &prism_dtu_demo_server::MultiOrgDemoConfig,
) -> anyhow::Result<()> {
    prism_dtu_demo_server::write_multi_admin_token_sidecar_to_path(
        servers,
        cfg,
        std::path::Path::new(TOKEN_MULTI_FILE),
    )
}

/// Wait for SIGINT or SIGTERM, then gracefully shut down all multi-org clone instances.
///
/// Mirrors `wait_for_shutdown_signal` but operates on `MultiInstanceServers` instead of
/// `DemoHarness`. Removes both the PID file and the nested URL sidecar on shutdown.
async fn wait_for_shutdown_signal_multi(servers: &prism_dtu_demo_server::MultiInstanceServers) {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};

        // SAFETY: failure to install a signal handler is a fatal setup error; panic is correct.
        #[allow(clippy::expect_used)]
        let mut sigterm =
            signal(SignalKind::terminate()).expect("failed to install SIGTERM handler");

        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                tracing::info!("start-multi: Received SIGINT — initiating graceful shutdown");
            }
            _ = sigterm.recv() => {
                tracing::info!("start-multi: Received SIGTERM — initiating graceful shutdown");
            }
        }
    }

    #[cfg(not(unix))]
    {
        // SAFETY: failure to install a signal handler is a fatal setup error; panic is correct.
        #[allow(clippy::expect_used)]
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl-C handler");
        tracing::info!("start-multi: Received Ctrl-C — initiating graceful shutdown");
    }

    // Send graceful shutdown signal to all instances.
    servers.shutdown();

    // Remove sidecar files.
    let _ = std::fs::remove_file(PID_FILE);
    let _ = std::fs::remove_file(URL_MULTI_FILE);
    // T-09: Remove admin-token sidecar alongside URL sidecar on shutdown.
    let _ = std::fs::remove_file(TOKEN_MULTI_FILE);

    tracing::info!("start-multi: All instances signalled for graceful shutdown.");
}

// `build_multi_clone_factory` and `start_multi_for_config` live in
// `prism_dtu_demo_server::multi_org_cmd` (library crate) so that integration tests
// in `tests/multi_org.rs` can access them. `cmd_start_multi` delegates to them.
// See `src/multi_org_cmd.rs` for the implementations.

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
    // TD-VSDD-060 sibling sweep (AC-004): all POST /dtu/configure call sites in client code.
    //
    // Enumerated in DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001 §Root Cause sibling table:
    // | Site                                             | X-Admin-Token? | Status          |
    // |--------------------------------------------------|----------------|-----------------|
    // | cmd_configure() — this function (main.rs)              | YES (FIXED)    | DEFECT → FIXED  |
    // | ac_3_configure_called_on_clone_port_directly           | YES            | Correct         |
    // | ac_3_no_harness_proxy_for_configure                    | YES            | Correct         |
    // | prism-dtu-crowdstrike td_wv0_07_*                      | YES            | Correct         |
    // | prism-dtu-{claroty,cyberint,armis,...} td_wv0_07_*     | YES            | Correct         |
    // | bc_2_06_019_scenario_progression.rs                    | YES            | Correct         |
    // | bc_3_6_001_ops_clone_failure_modes.rs (configure_failure helper) | YES | Correct (via Harness::admin_token_for()) |
    // | review_2026_06_10_deny_unknown.rs                      | YES            | Correct (via Harness::admin_token_for()) |
    // | prism-dtu-harness/src/builder.rs                       | N/A            | Synthetic (hung-socket; verifies client |
    // |   (test_build_harness_http_client_timeout_is_load_bearing) |             |   timeout, no DTU handler — not applicable) |
    //
    // rg 'dtu/configure' crates/ --type rust: 449 hits total (116 .post(…) client calls,
    // 21 .route(…) server registrations, remainder in doc comments and const strings).
    //
    // Only cmd_configure() was missing the header before this fix. All authenticated callers
    // use `clone.admin_token()` (prism-dtu-{sensor} tests) or `Harness::admin_token_for()`
    // (prism-dtu-harness tests) per ADR-003 Amendment #5. The builder.rs synthetic entry
    // POSTs to a hung socket with no DTU handler — authentication is not applicable there.

    // HIGH-1 fix: resolve the clone URL from whichever sidecar exists.
    //
    // `start` writes URL_FILE (flat: {name: url}).
    // `start-multi` writes URL_MULTI_FILE (nested: {org_slug: {sensor_id: url}}).
    //
    // `resolve_configure_url` tries the flat sidecar first; if absent, falls back to the
    // nested sidecar and accepts both full `{org_slug}-{sensor_id}` keys (e.g.
    // "org-b-cyberint") and bare sensor names (e.g. "cyberint" — EC-007 recovery form,
    // works when only one org has that sensor).
    let configure_url = prism_dtu_demo_server::resolve_configure_url(
        &clone_name,
        Some(std::path::Path::new(URL_FILE)),
        Some(std::path::Path::new(URL_MULTI_FILE)),
    )
    .map_err(|e| {
        anyhow::anyhow!(
            "configure: could not resolve URL for clone '{}': {}",
            clone_name,
            e
        )
    })?;

    // T-08 (DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001): resolve admin token from token sidecar.
    //
    // `start` writes TOKEN_FILE (flat: {name: token}).
    // `start-multi` writes TOKEN_MULTI_FILE (nested: {org_slug: {sensor_id: token}}).
    //
    // `resolve_configure_token` uses the same flat-first/nested-fallback/bare-name-disambiguation
    // logic as `resolve_configure_url`. Returns E-DEMO-007 if the token cannot be resolved.
    //
    // AD-017 credential safety: the token value is NOT logged (only token_present=true).
    let admin_token = prism_dtu_demo_server::resolve_configure_token(
        &clone_name,
        Some(std::path::Path::new(TOKEN_FILE)),
        Some(std::path::Path::new(TOKEN_MULTI_FILE)),
    )?;
    tracing::debug!(
        clone = %clone_name,
        token_present = true,
        "cmd_configure: resolved admin token from token sidecar"
    );

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to build HTTP client: {}", e))?;

    let resp = client
        .post(&configure_url)
        .header("Content-Type", "application/json")
        .header("X-Admin-Token", &admin_token)
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
