//! Signal handlers for the `prism` binary.
//!
//! Implements SIGTERM and SIGHUP tokio signal handlers per ADR-022 §B step 11
//! and BC-2.10.010 (Graceful Shutdown on SIGTERM/SIGINT).
//!
//! # SIGTERM (AC-6; BC-2.10.010)
//!
//! On SIGTERM:
//! 1. Emit `tracing::info!("Received SIGTERM — shutting down")`.
//! 2. Send on `shutdown_tx` broadcast channel to notify all subsystems.
//! 3. Drain in-flight queries.
//! 4. Close MCP server.
//! 5. Flush audit buffer.
//! 6. Close RocksDB.
//! 7. Exit 0.
//!
//! # SIGHUP
//!
//! On SIGHUP:
//! 1. Send on `reload_tx` mpsc channel.
//! 2. The reload consumer (HotReloadWatcher step 10) processes the reload.
//!    This path is a `todo!()` until S-1.12-FOLLOWUP wires HotReloadWatcher.
//!
//! # Platform Note
//!
//! `tokio::signal::unix` is only available on Unix platforms. The Windows
//! `Ctrl-C` handler covers the shutdown case on Windows (if ever supported).

use tokio::sync::{broadcast, mpsc};

/// Install the SIGTERM handler.
///
/// Waits for SIGTERM and broadcasts on `shutdown_tx` to initiate graceful
/// shutdown across all subsystem tasks.
///
/// Contract (BC-2.10.010): MUST emit `tracing::info!("Received SIGTERM — shutting down")`
/// before sending on `shutdown_tx`. MUST flush the audit buffer before `exit(0)`.
///
/// AC-6: Given SIGTERM delivered, process emits the SIGTERM log entry and exits 0.
pub async fn install_sigterm_handler(_shutdown_tx: broadcast::Sender<()>) {
    todo!("S-WAVE5-PREP-01: await tokio::signal::ctrl_c() + SIGTERM; emit 'Received SIGTERM — shutting down' tracing::info!; send on shutdown_tx; flush audit buffer; exit(0) per BC-2.10.010 and AC-6")
}

/// Install the SIGHUP handler.
///
/// Waits for SIGHUP and sends on `reload_tx` to trigger a config reload.
/// The consumer of `reload_tx` is the HotReloadWatcher task installed in
/// step 10 — that consumer is a `todo!()` until S-1.12-FOLLOWUP.
///
/// The reload path is idempotent: SIGHUP during an in-progress filesystem-
/// triggered reload is safe (both call the same `try_reload` path per ADR-022 §E).
pub async fn install_sighup_handler(_reload_tx: mpsc::Sender<()>) {
    todo!("S-WAVE5-PREP-01: await SIGHUP on unix platforms; send on reload_tx; SIGHUP reload consumer is todo!() until S-1.12-FOLLOWUP wires HotReloadWatcher")
}
