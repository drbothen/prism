//! `HarnessError` — error variants for the DTU test harness.
//!
//! Every public error variant maps to a documented behavioral contract clause:
//! - `UnknownOrg`     — BC-3.6.001 EC-001, BC-3.5.001 EC-001
//! - `UnknownDtuType` — BC-3.6.001 EC-002
//! - `PortConflict`   — BC-3.5.001 EC-003
//! - `StartupTimeout` — BC-3.5.001 EC-005, postcondition 5 / D-058
//! - `PortExhausted`  — BC-3.5.001 EC-003 (OS-level fallback)
//! - `CloneCrashed`   — BC-3.6.002 postconditions 1-5

use crate::types::DtuType;
use prism_core::ids::OrgId;

/// Errors returned by `Harness` operations and `HarnessBuilder::build()`.
///
/// All variants are `#[non_exhaustive]` to allow adding diagnostic fields in
/// future waves without breaking existing match arms in downstream test crates.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum HarnessError {
    /// The caller referenced an `org_slug` that was never registered with the
    /// harness via `HarnessBuilder::with_customer`.
    ///
    /// No HTTP request is sent and no side effects occur.
    /// (BC-3.6.001 EC-001; BC-3.5.001 EC-001)
    #[error("unknown org slug: {slug:?}")]
    UnknownOrg { slug: String },

    /// The caller referenced a `DtuType` that is not registered for the given
    /// org in this harness instance.
    ///
    /// No HTTP request is sent and no side effects occur.
    /// (BC-3.6.001 EC-002)
    #[error("unknown DTU type {dtu_type:?} for org {slug:?}")]
    UnknownDtuType { slug: String, dtu_type: String },

    /// Two or more clones attempted to bind the same address, or a clone could
    /// not bind its assigned port (OS EADDRINUSE).
    ///
    /// No partial `Harness` is returned; all partially-started tasks are aborted.
    /// (BC-3.5.001 EC-003)
    #[error("port conflict for org {org:?} DTU {dtu:?}")]
    PortConflict { org: OrgId, dtu: DtuType },

    /// The 12-clone parallel startup exceeded the 200ms wall-clock budget.
    ///
    /// All partially-started tasks are aborted before this error is returned.
    /// (BC-3.5.001 EC-005; D-058 locked decision)
    #[error("harness startup timed out (200ms budget exceeded)")]
    StartupTimeout,

    /// The OS could not provide an ephemeral loopback port for a new clone.
    ///
    /// This is distinct from `PortConflict` — it indicates OS-level port
    /// exhaustion rather than a specific bind collision.
    /// (BC-3.5.001 Invariant 4)
    #[error("OS ephemeral port pool exhausted; could not bind a new clone")]
    PortExhausted,

    /// The clone task for `(org_id, dtu_type)` exited unexpectedly — due to a
    /// panic, returning `Err`, or completing `Ok` before the test finished.
    ///
    /// `cause` is the panic message string, the `Display` of the `Err` value,
    /// or `"task exited Ok before test completion"` for premature clean exit.
    ///
    /// If the panic payload was not a `&str` or `String`, `cause` is
    /// `"(non-string panic payload)"` per BC-3.6.002 Invariant 4.
    ///
    /// (BC-3.6.002 postconditions 1-5; BC-3.6.001 EC-004)
    #[error("clone crashed for org {org_id} DTU {dtu_type:?}: {cause}")]
    CloneCrashed {
        org_id: OrgId,
        dtu_type: DtuType,
        cause: String,
    },

    /// An I/O error occurred while binding a `TcpListener` during `build()`.
    #[error("I/O error during harness build: {0}")]
    Io(#[from] std::io::Error),

    /// An HTTP error occurred while sending a `POST /dtu/configure` request.
    ///
    /// Wraps `reqwest::Error`; indicates a network-level failure communicating
    /// with an in-process clone's admin endpoint.
    #[error("HTTP error during failure injection: {0}")]
    Http(#[from] reqwest::Error),
}
