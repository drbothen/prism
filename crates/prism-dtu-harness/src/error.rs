//! `HarnessError` — error variants for the DTU test harness.
//!
//! Every public error variant maps to a documented behavioral contract clause:
//! - `UnknownOrg`              — BC-3.6.001 EC-001, BC-3.5.001 EC-001
//! - `UnknownDtuType`          — BC-3.6.001 EC-002
//! - `PortConflict`            — BC-3.5.001 EC-003
//! - `StartupTimeout`          — BC-3.5.001 EC-005, postcondition 5 / D-058
//! - `PortExhausted`           — BC-3.5.001 EC-003 (OS-level fallback)
//! - `CloneCrashed`            — BC-3.6.002 postconditions 1-5
//! - `NetworkPortAllocation`   — BC-3.5.002 EC-004 (Network mode bind failure)

use prism_core::ids::OrgId;

use crate::types::DtuType;

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

    /// Network-mode pre-allocation failed: the OS could not bind one or more
    /// of the simultaneous `TcpListener`s required for `IsolationMode::Network`.
    ///
    /// Distinct from `PortConflict` (a named-port EADDRINUSE collision) and
    /// `PortExhausted` (OS ephemeral pool exhaustion): this variant captures
    /// `io::Error` from the simultaneous bind phase of Network-mode `build()`.
    ///
    /// No partial `Harness` is returned; all pre-allocated listeners are dropped.
    /// (BC-3.5.002 EC-004; ADR-011 §2.5 pre-allocation strategy; D-058)
    #[error("network-mode port pre-allocation failed: {source}")]
    NetworkPortAllocation { source: std::io::Error },

    /// An I/O error occurred while binding a `TcpListener` during `build()`.
    #[error("I/O error during harness build: {0}")]
    Io(#[from] std::io::Error),

    /// An HTTP error occurred while sending a `POST /dtu/configure` request.
    ///
    /// Wraps `reqwest::Error`; indicates a network-level failure communicating
    /// with an in-process clone's admin endpoint.
    #[error("HTTP error during failure injection: {0}")]
    Http(#[from] reqwest::Error),

    /// Two [`crate::multi_instance::HarnessEntry`] items share the same
    /// `(org_slug, sensor_id)` pair.
    ///
    /// Returned immediately before any clone instance is started. Silent
    /// last-wins behaviour is forbidden — duplicate entries indicate test-code
    /// misconfiguration that must be surfaced immediately.
    /// (BC-2.06.017 Postcondition 7 / EC-017-003)
    #[error("duplicate harness entry for org_slug={org_slug:?} sensor_id={sensor_id:?}")]
    DuplicateKey {
        /// The org slug from the conflicting entry.
        org_slug: String,
        /// The sensor ID from the conflicting entry.
        sensor_id: String,
    },

    /// One or more clone bind operations failed during
    /// [`crate::multi_instance::MultiInstanceHarness::start`].
    ///
    /// All bind operations are attempted before this error is returned
    /// (multi-error aggregation per BC-2.06.017 INV-ERR-003-COMPAT).
    /// Successfully-started instances are shut down before the error is
    /// returned (no zombie instances).
    /// (BC-2.06.017 Postcondition 6)
    #[error("one or more clone bind operations failed ({} failures)", .0.len())]
    BindFailure(Vec<BindError>),
}

/// A single clone bind failure within a
/// [`HarnessError::BindFailure`] error.
///
/// (BC-2.06.017 Amendment 2 — name disambiguation from the demo-server-side
/// `DemoBindError` type which uses `instance_name` instead of `org_slug`/`sensor_id`)
#[derive(Debug)]
#[non_exhaustive]
pub struct BindError {
    /// The org slug of the entry that failed to bind.
    pub org_slug: String,
    /// The sensor ID of the entry that failed to bind.
    pub sensor_id: String,
    /// The underlying OS bind error.
    pub source: std::io::Error,
}

impl std::fmt::Display for BindError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "bind failed for org_slug={:?} sensor_id={:?}: {}",
            self.org_slug, self.sensor_id, self.source
        )
    }
}

impl std::error::Error for BindError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}
