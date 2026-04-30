//! `Harness` — the running multi-tenant DTU test harness.
//!
//! `Harness` is the effectful shell returned by `HarnessBuilder::build().await`.
//! After construction, its `endpoints` map is immutable for the harness lifetime
//! (BC-3.5.001 Invariant 2).
//!
//! # Crash check contract
//!
//! Every method that targets a specific `(OrgId, DtuType)` MUST call
//! `self.check_crash(org_id, dtu_type)?` before any HTTP connection attempt.
//! This converts silent `ConnectionRefused` errors into explicit `CloneCrashed`
//! diagnostics (BC-3.6.002 Invariant 1).
//!
//! # Drop behavior
//!
//! `impl Drop for Harness` sends shutdown signals to all non-crashed clones,
//! waits up to 5s for graceful exit, then calls `BehavioralClone::stop()` (hard
//! abort) on any that have not exited within the grace period (BC-3.5.001 EC-004).
//!
//! # Architecture Anchors
//!
//! - ADR-011 §2.2  — `(OrgId, DtuType)` keyed struct fields
//! - ADR-011 §2.5  — teardown: graceful shutdown + hard abort fallback
//! - ADR-011 §2.6  — crash detection: `JoinHandle` monitoring
//! - ADR-011 §2.7  — failure injection: `inject_failure`, `clear_failure`
//! - BC-3.5.001 Invariants 1-4; postconditions 1-5
//! - BC-3.6.001 Invariants 1-4; postconditions 1-4
//! - BC-3.6.002 Invariants 1-4; postconditions 1-5

use std::collections::HashMap;
use std::net::SocketAddr;

use tokio::sync::watch;
use tokio::task::JoinHandle;

use crate::builder::HarnessBuilder;
use crate::crash_monitor::poll_crash;
use crate::error::HarnessError;
use crate::types::{DtuType, OrgKey};
use prism_core::ids::OrgId;
use prism_dtu_common::config::FailureMode;

/// The running multi-tenant DTU test harness.
///
/// Returned by `HarnessBuilder::build().await`. Holds all per-clone state
/// including the immutable `endpoints` map and the crash/shutdown channels.
///
/// (Story S-3.3.03, Task 5; BC-3.5.001; BC-3.6.001; BC-3.6.002)
// S-3.3.03 stub: fields are populated by build() (not yet implemented).
// Suppress dead_code for the stub phase — all fields will be read in implementation.
#[allow(dead_code)]
pub struct Harness {
    /// Immutable map of `(OrgId, DtuType)` → bound `SocketAddr`.
    ///
    /// Populated atomically during `build()` and never modified thereafter.
    /// (BC-3.5.001 Invariant 2)
    pub(crate) endpoints: HashMap<OrgKey, SocketAddr>,

    /// Per-clone crash notification channels.
    ///
    /// Each receiver holds `None` (healthy) or `Some(cause)` (crashed).
    /// Checked before every clone-targeted operation via `poll_crash`.
    /// (BC-3.6.002 Invariants 1-3)
    pub(crate) crash_channels: HashMap<OrgKey, watch::Receiver<Option<String>>>,

    /// Per-clone graceful-shutdown senders.
    ///
    /// Consumed in `Drop::drop()` to signal shutdown to running clones.
    /// (BC-3.5.001 postcondition 4; EC-004)
    pub(crate) shutdown_senders: HashMap<OrgKey, tokio::sync::broadcast::Sender<()>>,

    /// Per-clone `JoinHandle`s for awaiting task completion in `Drop`.
    ///
    /// (ADR-011 §2.6; BC-3.6.002 postcondition 4)
    pub(crate) task_handles: HashMap<OrgKey, JoinHandle<()>>,

    /// Admin tokens indexed by `OrgKey`, used for `POST /dtu/configure` auth.
    ///
    /// Retrieved from `BehavioralClone::admin_token()` after `start_on()`.
    /// (ADR-003 Amendment §5; BC-3.6.001 Invariant 3)
    pub(crate) admin_tokens: HashMap<OrgKey, String>,

    /// `reqwest` client for sending `POST /dtu/configure` requests.
    ///
    /// One shared client per harness — clone admin endpoints are all loopback,
    /// so connection pooling is efficient.
    pub(crate) http_client: reqwest::Client,
}

impl Harness {
    /// Create a `HarnessBuilder` for constructing a new harness.
    ///
    /// Equivalent to `HarnessBuilder::new()`. Provided as an ergonomic entry point.
    ///
    /// ```ignore
    /// let harness = Harness::builder()
    ///     .isolation(IsolationMode::Logical)
    ///     .with_customer("acme-corp")
    ///     .build()
    ///     .await?;
    /// ```
    pub fn builder() -> HarnessBuilder {
        HarnessBuilder::new()
    }

    /// Return the immutable endpoints map.
    ///
    /// Maps `(OrgId, DtuType)` to the `SocketAddr` each clone is bound to.
    /// Contains exactly `|orgs| × |dtu_types_per_org|` entries after `build()`.
    ///
    /// (BC-3.5.001 postcondition 1; Invariant 2; VP-122)
    pub fn endpoints(&self) -> &HashMap<OrgKey, SocketAddr> {
        &self.endpoints
    }

    /// Check whether the clone at `(org_id, dtu_type)` has crashed.
    // S-3.3.03 stub: called from inject_failure/clear_failure once implemented.
    #[allow(dead_code)]
    ///
    /// Returns `Err(HarnessError::CloneCrashed { .. })` if a crash was detected,
    /// `Ok(())` otherwise.
    ///
    /// This is the core enforcement point for BC-3.6.002 Invariant 1 — every
    /// clone-targeted operation calls this helper before any HTTP attempt.
    ///
    /// (BC-3.6.002 Invariants 1, 3)
    fn check_crash(&self, org_id: OrgId, dtu_type: DtuType) -> Result<(), HarnessError> {
        let key = (org_id, dtu_type);
        if let Some(rx) = self.crash_channels.get(&key) {
            if let Some(cause) = poll_crash(rx) {
                return Err(HarnessError::CloneCrashed {
                    org_id,
                    dtu_type,
                    cause,
                });
            }
        }
        Ok(())
    }

    /// Resolve `(slug, dtu_type)` to `(OrgId, SocketAddr)`, checking for unknown org/DTU.
    // S-3.3.03 stub: called from inject_failure/clear_failure once implemented.
    #[allow(dead_code)]
    ///
    /// Returns `Err(UnknownOrg)` if the slug is not in `endpoints`,
    /// `Err(UnknownDtuType)` if the DTU type is not registered for that org.
    ///
    /// (BC-3.6.001 EC-001, EC-002)
    fn resolve_endpoint(
        &self,
        _slug: &str,
        _dtu_type: DtuType,
    ) -> Result<(OrgId, SocketAddr), HarnessError> {
        todo!(
            "S-3.3.03 implementation: scan endpoints keys for matching org_slug (requires \
             reverse slug→OrgId lookup — store a slug→OrgId side-map in Harness or iterate \
             endpoints keys). Return Err(UnknownOrg) or Err(UnknownDtuType) as appropriate."
        )
    }

    /// Inject a failure mode into the clone at `(org_slug, dtu_type)`.
    ///
    /// # Preconditions
    ///
    /// - The `(org_slug, dtu_type)` pair must be registered in the harness.
    /// - The target clone must not have crashed (checked first via `check_crash`).
    ///
    /// # Behavior
    ///
    /// Sends `POST http://{addr}/dtu/configure` with the serialized `FailureMode`
    /// and the clone's `admin_token` in the `X-Admin-Token` header.
    ///
    /// Returns `Ok(())` on HTTP 200. HTTP errors are mapped to `HarnessError::Http`.
    ///
    /// `FailureMode::None` is equivalent to `clear_failure` (BC-3.6.001 Invariant 4).
    ///
    /// (BC-3.6.001 postconditions 1-4; Invariant 2; EC-001 through EC-007)
    pub async fn inject_failure(
        &self,
        _slug: &str,
        _dtu_type: DtuType,
        _mode: FailureMode,
    ) -> Result<(), HarnessError> {
        todo!(
            "S-3.3.03 implementation: \
             (1) resolve_endpoint(slug, dtu_type) → (org_id, addr); \
             (2) check_crash(org_id, dtu_type)?; \
             (3) handle EC-007: if mode == FailureMode::NetworkTimeout {{ after_ms: 0 }}, \
                 treat as FailureMode::None; \
             (4) POST http://{{addr}}/dtu/configure with admin_token header and JSON body; \
             (5) return Ok(()) on HTTP 200, Err(Http(_)) otherwise. \
             See BC-3.6.001 postcondition clause 1; ADR-011 §2.7."
        )
    }

    /// Clear any injected failure from the clone at `(org_slug, dtu_type)`.
    ///
    /// Equivalent to `inject_failure(slug, dtu_type, FailureMode::None)`.
    ///
    /// Idempotent: calling this when no failure is active returns `Ok(())`
    /// with no state change (BC-3.6.001 postcondition 4; EC-006).
    ///
    /// (BC-3.6.001 postconditions 3, 4; Invariant 4)
    pub async fn clear_failure(&self, _slug: &str, _dtu_type: DtuType) -> Result<(), HarnessError> {
        todo!(
            "S-3.3.03 implementation: delegate to inject_failure(slug, dtu_type, FailureMode::None). \
             See BC-3.6.001 postcondition 3; Invariant 4."
        )
    }
}

impl Drop for Harness {
    /// Tear down all clone tasks gracefully, with a 5-second hard-abort fallback.
    ///
    /// Steps:
    /// 1. Send shutdown signal via `shutdown_senders` to all running clones.
    /// 2. Await each `JoinHandle` with a 5-second timeout.
    /// 3. On timeout: call `handle.abort()` (hard abort).
    ///
    /// Already-crashed clones (task exited) are no-ops — their `JoinHandle`
    /// completes immediately (BC-3.6.002 postcondition 4).
    ///
    /// (BC-3.5.001 postcondition 4; EC-003, EC-004; BC-3.6.002 postcondition 4; VP-124)
    fn drop(&mut self) {
        // Implementation note: Drop cannot be async. Use `Handle::current()` to
        // spawn a blocking join, or use `JoinHandle::abort()` directly after
        // sending shutdown signals synchronously.
        //
        // The standard pattern for test harness teardown is:
        //   for sender in self.shutdown_senders.drain() { let _ = sender.send(()); }
        //   for handle in self.task_handles.drain() {
        //       handle.abort(); // immediate hard-abort is acceptable in drop
        //   }
        //
        // A graceful-then-abort approach requires spawning a detached task:
        //   tokio::spawn(async { /* await with timeout then abort */ });
        //
        // The implementer must choose between these strategies per ADR-011 §2.5.
        //
        // S-3.3.03 implementation: send all shutdown signals then abort all handles.
        // todo!() is intentionally NOT placed here — the compiler requires Drop to
        // be compilable even in stub form. The body is a no-op stub.
        //
        // Drain shutdown senders (signals graceful shutdown to clone tasks)
        for (_key, sender) in self.shutdown_senders.drain() {
            let _ = sender.send(());
        }
        // Abort all task handles (hard abort — acceptable in drop; graceful drain
        // is best-effort via the shutdown signal above)
        for (_key, handle) in self.task_handles.drain() {
            handle.abort();
        }
    }
}
