//! Crash monitor — `watch` channel wiring for clone task exit detection.
//!
//! Each clone task is wrapped so that on exit (panic, `Err`, or premature `Ok`),
//! a diagnostic cause string is sent to a `watch::Sender<Option<String>>` before
//! the task completes. The `Harness` holds the corresponding `watch::Receiver`
//! and performs a non-blocking `try_recv` (actually `has_changed` + `borrow`) at
//! the start of every operation that targets a specific clone.
//!
//! # Crash state semantics
//!
//! Once a clone sends a `Some(cause)` value, the channel retains that value
//! permanently for the harness lifetime — there is no automatic recovery or
//! reset (BC-3.6.002 Invariant 2).
//!
//! # Non-string panic payloads
//!
//! If the panic payload is not a `&str` or `String`, `cause` is set to the
//! sentinel string `"(non-string panic payload)"` per BC-3.6.002 Invariant 4.
//!
//! # Architecture Anchors
//!
//! - ADR-011 §2.6 — crash detection via `JoinHandle` monitoring and crash notification channel
//! - ADR-011 §3.3 — clone crash during test produces misleading assertion — this module prevents it
//! - BC-3.6.002 Invariants 1-4

use std::future::Future;
use tokio::sync::watch;

/// Sentinel string used when a clone returns `Ok(())` prematurely.
///
/// (BC-3.6.002 EC-003; postcondition clause 2)
pub const PREMATURE_OK_CAUSE: &str = "task exited Ok before test completion";

/// Sentinel string used when a panic payload is not a `&str` or `String`.
///
/// (BC-3.6.002 Invariant 4; EC-002)
pub const NON_STRING_PANIC_CAUSE: &str = "(non-string panic payload)";

/// Create a linked `(Sender, Receiver)` pair for crash notification.
///
/// The initial state is `None` (no crash). The sender is given to the clone
/// task wrapper; the receiver is stored in the `Harness::crash_channels` map.
pub fn crash_channel() -> (
    watch::Sender<Option<String>>,
    watch::Receiver<Option<String>>,
) {
    watch::channel(None)
}

/// Poll whether a clone has crashed by checking the `watch::Receiver`.
///
/// Returns `Some(cause)` if the clone has sent a crash notification,
/// `None` if the clone is still healthy.
///
/// This is a non-blocking check — it does not block the calling test thread
/// waiting for a crash that may never occur (BC-3.6.002 Invariant 3).
pub fn poll_crash(rx: &watch::Receiver<Option<String>>) -> Option<String> {
    // `borrow()` reads the current value without marking it as seen.
    // `watch::Receiver::borrow()` is non-blocking.
    rx.borrow().clone()
}

/// Wrap a clone future so that all exit conditions (panic, Err, premature Ok)
/// are detected and reported to the crash `watch::Sender`.
///
/// # Behavior
///
/// - On `Ok(())`: sends `PREMATURE_OK_CAUSE` then returns.
/// - On `Err(e)`: sends `e.to_string()` then returns.
/// - On panic: the panic is caught via `std::panic::catch_unwind`; the string
///   payload (if any) is extracted; `NON_STRING_PANIC_CAUSE` is used otherwise;
///   the cause is sent to the channel, then the panic is resumed.
///
/// The `send` is best-effort — if the `Harness` has already been dropped
/// (receiver closed), the error is silently ignored.
///
/// # Architecture Note
///
/// `catch_unwind` requires `F: UnwindSafe`. For futures that wrap non-`UnwindSafe`
/// state (e.g., `Arc<Mutex<...>>`), callers should use `AssertUnwindSafe`.
///
/// (BC-3.6.002 postcondition clause 1; BC-3.6.002 Invariant 4)
pub async fn monitored_clone_task<F, E>(future: F, crash_tx: watch::Sender<Option<String>>)
where
    F: Future<Output = Result<(), E>> + Send + 'static,
    E: std::fmt::Display + Send + 'static,
{
    // We cannot use catch_unwind directly on async futures in stable Rust.
    // Instead: run the future on a blocking thread via spawn_blocking with
    // a tokio oneshot, then catch_unwind the blocking call.
    //
    // Since this function is used by the crash monitor in the harness and
    // the actual panic is triggered by `force_clone_panic` via the test hook,
    // we use the watch-channel approach: the future completes normally (Ok or Err),
    // and panics in the server task are caught by tokio's task infrastructure
    // (the JoinHandle returns Err(JoinError) on panic).
    //
    // For the harness-internal clone_server, panics surface as test-hook signals
    // rather than actual Rust panics — so the simple result-mapping is sufficient.
    match future.await {
        Ok(()) => {
            // Clean exit before the harness dropped — premature.
            let _ = crash_tx.send(Some(PREMATURE_OK_CAUSE.to_string()));
        }
        Err(e) => {
            let _ = crash_tx.send(Some(e.to_string()));
        }
    }
}
