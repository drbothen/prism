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
pub async fn install_sigterm_handler(shutdown_tx: broadcast::Sender<()>) {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};

        let mut sigterm = match signal(SignalKind::terminate()) {
            Ok(s) => s,
            Err(e) => {
                tracing::error!(
                    "Failed to register SIGTERM handler: {e}; continuing without handler"
                );
                return;
            }
        };

        tokio::select! {
            _ = sigterm.recv() => {
                // BC-2.10.010: emit the required log line FIRST.
                tracing::info!("Received SIGTERM — shutting down");

                // Notify all subsystems via broadcast channel.
                // Errors here mean all receivers have been dropped — acceptable at shutdown.
                let _ = shutdown_tx.send(());

                // BC-2.10.010: flush audit buffer before exit.
                // In the full implementation (post S-3.02-FOLLOWUP-RUNTIME), this flushes
                // the RocksDB audit_buffer CF. For the chassis, the tracing subscriber
                // handles buffering and the OS flushes stdout/stderr on exit.
                tracing::info!(
                    "Audit buffer flush deferred to S-3.02-FOLLOWUP-RUNTIME (chassis only) — exiting cleanly"
                );

                // AC-6: exit 0 on clean SIGTERM shutdown.
                std::process::exit(0);
            }
            _ = tokio::signal::ctrl_c() => {
                // Handle Ctrl-C the same as SIGTERM for graceful shutdown.
                tracing::info!("Received SIGTERM — shutting down");
                let _ = shutdown_tx.send(());
                tracing::info!(
                    "Audit buffer flush deferred to S-3.02-FOLLOWUP-RUNTIME (chassis only) — exiting cleanly"
                );
                std::process::exit(0);
            }
        }
    }

    #[cfg(not(unix))]
    {
        // On non-Unix platforms (Windows), use Ctrl-C as the shutdown signal.
        if let Ok(()) = tokio::signal::ctrl_c().await {
            tracing::info!("Received SIGTERM — shutting down");
            let _ = shutdown_tx.send(());
            tracing::info!(
                    "Audit buffer flush deferred to S-3.02-FOLLOWUP-RUNTIME (chassis only) — exiting cleanly"
                );
            std::process::exit(0);
        } else {
            tracing::error!("Ctrl-C signal handler failed; continuing without SIGTERM handler");
        }
    }
}

/// Register the SIGTERM signal stream synchronously and return a future that,
/// when awaited, waits for the signal and performs graceful shutdown.
///
/// This split registration allows callers to guarantee the OS-level signal
/// handler is installed **before** writing a readiness sentinel — eliminating
/// the race window where SIGTERM arrives between sentinel write and handler
/// registration.
///
/// # Usage (test gate pattern)
/// ```ignore
/// let handler_fut = signals::create_sigterm_future(shutdown_tx);
/// // Signal handler is now registered (sync part complete).
/// std::fs::write(&sentinel_path, b"ready")?;  // safe to signal readiness
/// handler_fut.await;                           // blocks until SIGTERM
/// ```
///
/// On non-Unix platforms this function falls through to `install_sigterm_handler`.
#[cfg(unix)]
pub fn create_sigterm_future(
    shutdown_tx: broadcast::Sender<()>,
) -> impl std::future::Future<Output = ()> {
    use tokio::signal::unix::{signal, SignalKind};

    // Register the OS-level SIGTERM handler here, synchronously, before
    // returning the future.  Any SIGTERM delivered after this point will be
    // queued by the kernel and delivered when the future is first polled.
    let sigterm_result = signal(SignalKind::terminate());

    async move {
        let mut sigterm = match sigterm_result {
            Ok(s) => s,
            Err(e) => {
                tracing::error!(
                    "Failed to register SIGTERM handler: {e}; continuing without handler"
                );
                return;
            }
        };

        tokio::select! {
            _ = sigterm.recv() => {
                // BC-2.10.010: emit the required log line FIRST.
                tracing::info!("Received SIGTERM — shutting down");
                let _ = shutdown_tx.send(());
                tracing::info!(
                    "Audit buffer flush deferred to S-3.02-FOLLOWUP-RUNTIME (chassis only) — exiting cleanly"
                );
                std::process::exit(0);
            }
            _ = tokio::signal::ctrl_c() => {
                tracing::info!("Received SIGTERM — shutting down");
                let _ = shutdown_tx.send(());
                tracing::info!(
                    "Audit buffer flush deferred to S-3.02-FOLLOWUP-RUNTIME (chassis only) — exiting cleanly"
                );
                std::process::exit(0);
            }
        }
    }
}

/// Install the SIGHUP handler.
///
/// Waits for SIGHUP and sends on `reload_tx` to trigger a config reload.
/// The consumer of `reload_tx` is the HotReloadWatcher task installed in
/// step 10 — that consumer is a `todo!()` until S-1.12-FOLLOWUP.
///
/// The reload path is idempotent: SIGHUP during an in-progress filesystem-
/// triggered reload is safe (both call the same `try_reload` path per ADR-022 §E).
pub async fn install_sighup_handler(reload_tx: mpsc::Sender<()>) {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};

        let mut sighup = match signal(SignalKind::hangup()) {
            Ok(s) => s,
            Err(e) => {
                tracing::error!(
                    "Failed to register SIGHUP handler: {e}; continuing without handler"
                );
                return;
            }
        };

        loop {
            sighup.recv().await;
            tracing::info!("Received SIGHUP — triggering config reload");

            // Send on reload_tx; the HotReloadWatcher consumer (step 10) processes it.
            // If the consumer has been dropped (e.g., watcher not yet started),
            // the send fails silently (acceptable — reload is best-effort).
            if reload_tx.send(()).await.is_err() {
                tracing::warn!(
                    "SIGHUP received but reload channel has no consumer \
                     (HotReloadWatcher not yet installed — deferred to S-1.12-FOLLOWUP)"
                );
            }
        }
    }

    #[cfg(not(unix))]
    {
        // SIGHUP is not available on Windows; this handler is a no-op.
        tracing::debug!("SIGHUP handler: no-op on non-Unix platforms");
        // Keep the reload_tx alive to prevent premature channel close.
        let _ = reload_tx;
    }
}
