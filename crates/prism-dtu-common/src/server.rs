//! Shared HTTP server helpers for DTU clone lifecycle management (S-PERF-GATE-005).
//!
//! Provides `spawn_with_internal_shutdown` for starting a plain-HTTP server with a
//! built-in broadcast-channel graceful-shutdown signal, enabling `clone.stop()` to
//! complete in < 10ms for idle servers rather than waiting for the 5s hard-abort
//! fallback.
//!
//! Provides `graceful_stop` to fire the signal and await the `JoinHandle`.

use std::{convert::Infallible, time::Duration};

use tokio::{sync::broadcast, task::JoinHandle};

/// Spawn an HTTP server with an internal graceful-shutdown broadcast channel.
///
/// Creates a `broadcast::channel::<()>(1)`, wires the receiver into
/// `axum::serve(listener, make_service).with_graceful_shutdown(...)`, spawns the
/// task, and returns `(JoinHandle<()>, broadcast::Sender<()>)`.
///
/// The caller stores the `Sender` in `self.internal_shutdown_tx` and calls
/// `graceful_stop` from `stop()` to signal and await the server task.
///
/// # Generic bounds
///
/// `M` and `S` mirror `axum::serve<M, S>`'s own trait bounds exactly (plus
/// `Send + 'static` for task spawning), so this helper accepts both plain
/// `axum::Router<()>` and wrapped make-services — for example the
/// `NormalizePathLayer`-wrapped variant used by `ClarotyClone` to work around
/// axum#2377 trailing-slash routing.
///
/// # Only for the HTTP (`shutdown = None`) path
///
/// The harness-provided-shutdown path (`shutdown = Some(rx)`) is NOT handled here;
/// call sites keep their existing `with_graceful_shutdown(rx)` pattern unchanged.
/// The TLS path continues to use `axum_server::Handle::graceful_shutdown`.
pub fn spawn_with_internal_shutdown<M, S>(
    listener: tokio::net::TcpListener,
    make_service: M,
    error_context: &'static str,
) -> (JoinHandle<()>, broadcast::Sender<()>)
where
    M: for<'a> tower::Service<axum::serve::IncomingStream<'a>, Error = Infallible, Response = S>
        + Send
        + 'static,
    for<'a> <M as tower::Service<axum::serve::IncomingStream<'a>>>::Future: Send,
    S: tower::Service<
            axum::extract::Request,
            Response = axum::response::Response,
            Error = Infallible,
        > + Clone
        + Send
        + 'static,
    S::Future: Send,
{
    let (tx, mut rx) = broadcast::channel::<()>(1);
    let handle = tokio::spawn(async move {
        // SAFETY: server task crash must surface immediately as a fatal error.
        #[allow(clippy::expect_used)]
        axum::serve(listener, make_service)
            .with_graceful_shutdown(async move {
                let _ = rx.recv().await;
            })
            .await
            .expect(error_context);
    });
    (handle, tx)
}

/// Fire an optional internal shutdown signal and await the server task.
///
/// Sends `()` on `sender` (if `Some`) to trigger graceful drain, then waits
/// up to `fallback` for the task to complete before hard-aborting.
///
/// For idle servers the drain completes in < 10ms; the `fallback` bound (~250ms)
/// is a safety net for slow-draining connections.
///
/// Called from each clone's `stop()` for both the HTTP internal path (sender
/// is `Some`) and the harness/TLS paths (sender is `None`; harness already
/// fired its own signal or `axum_server::Handle::graceful_shutdown` was called).
pub async fn graceful_stop(
    sender: Option<broadcast::Sender<()>>,
    mut handle: JoinHandle<()>,
    fallback: Duration,
) {
    if let Some(tx) = sender {
        let _ = tx.send(());
    }
    tokio::select! {
        _ = &mut handle => {
            // Server task completed gracefully.
        }
        _ = tokio::time::sleep(fallback) => {
            // Drain window expired — hard-abort the server task.
            handle.abort();
        }
    }
}
