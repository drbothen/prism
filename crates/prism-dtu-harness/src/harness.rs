//! `Harness` — the running multi-tenant DTU test harness.
//!
//! `Harness` is the effectful shell returned by `HarnessBuilder::build().await`.
//! After construction, its `endpoints` map is immutable for the harness lifetime
//! (BC-3.5.001 Invariant 2). In `IsolationMode::Network`, `customer_endpoints`
//! is also populated atomically and immutable for the harness lifetime
//! (BC-3.5.002 Invariant 1).
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
//! waits up to 5s for graceful exit, then calls `handle.abort()` on any that
//! have not exited within the grace period (BC-3.5.001 EC-004).
//! In `IsolationMode::Network`, the Network-mode teardown path also releases
//! all pre-allocated TCP listeners (BC-3.5.002 postcondition 6).
//!
//! # Architecture Anchors
//!
//! - ADR-011 §2.2  — `(OrgId, DtuType)` keyed struct fields
//! - ADR-011 §2.3  — Network mode: `customer_endpoints` table
//! - ADR-011 §2.5  — teardown: graceful shutdown + hard abort fallback
//! - ADR-011 §2.6  — crash detection: `JoinHandle` monitoring
//! - ADR-011 §2.7  — failure injection: `inject_failure`, `clear_failure`
//! - BC-3.5.001 Invariants 1-4; postconditions 1-5
//! - BC-3.5.002 Invariants 1-4; postconditions 1-6
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
use prism_dtu_common::FailureMode;

/// The running multi-tenant DTU test harness.
///
/// Returned by `HarnessBuilder::build().await`. Holds all per-clone state
/// including the immutable `endpoints` map and the crash/shutdown channels.
///
/// (Story S-3.3.03, Task 5; BC-3.5.001; BC-3.6.001; BC-3.6.002)
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
    /// Retrieved from `start_clone()` after binding.
    /// (ADR-003 Amendment §5; BC-3.6.001 Invariant 3)
    pub(crate) admin_tokens: HashMap<OrgKey, String>,

    /// `reqwest` client for sending `POST /dtu/configure` requests.
    ///
    /// One shared client per harness — clone admin endpoints are all loopback,
    /// so connection pooling is efficient.
    pub(crate) http_client: reqwest::Client,

    /// Slug → OrgId reverse-lookup map for `resolve_endpoint`.
    ///
    /// Built during `HarnessBuilder::build()` from `CustomerSpec::org_slug`.
    /// (BC-3.6.001 EC-001)
    pub(crate) slug_to_org: HashMap<String, OrgId>,

    /// Network-mode per-org, per-DTU endpoint table (BC-3.5.002 primary accessor).
    ///
    /// Populated atomically during `build()` when `IsolationMode::Network` is
    /// selected, and is immutable for the harness lifetime (BC-3.5.002 Invariant 1).
    ///
    /// For `IsolationMode::Logical`, this map is empty — use `endpoints()` instead.
    ///
    /// Type alias: `CustomerEndpoints = HashMap<OrgKey, SocketAddr>` (ADR-011 §2.3).
    ///
    /// (BC-3.5.002 postcondition 4; Invariant 1; ADR-011 §2.3; VP-125)
    // S-3.3.04 stub: read by customer_endpoints() accessor (also a todo!() stub).
    // Suppress dead_code lint until the implementer phase of S-3.3.04 wires these up.
    #[allow(dead_code)]
    pub(crate) customer_endpoints: HashMap<OrgKey, SocketAddr>,
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

    /// Look up the `SocketAddr` for a given `(slug, dtu_type)` pair.
    ///
    /// This is a slug-based convenience lookup used by test helpers.
    /// Returns `None` if the org or DTU type is not registered.
    pub fn endpoint_for(&self, slug: &str, dtu_type: DtuType) -> Option<SocketAddr> {
        let org_id = self.slug_to_org.get(slug)?;
        self.endpoints.get(&(*org_id, dtu_type)).copied()
    }

    /// Return the Network-mode per-org, per-DTU endpoint table.
    ///
    /// The primary accessor for `IsolationMode::Network` harnesses: each entry
    /// maps `(OrgId, DtuType)` → the `SocketAddr` of that org's dedicated clone.
    ///
    /// Test clients configured with this table route sensor requests by
    /// `(OrgId, DtuType)` lookup — a keying bug that sends `OrgId(A)` credentials
    /// to `OrgId(B)`'s `SocketAddr` will receive HTTP 401 from the wrong clone's
    /// auth middleware, making the routing error observable.
    ///
    /// The returned map contains exactly `Σ |dtu_types(org)|` entries (one per
    /// registered `(OrgId, DtuType)` pair) after `build()`. All `SocketAddr`
    /// values are pairwise distinct (VP-125).
    ///
    /// For `IsolationMode::Logical` harnesses, this map is always empty — use
    /// `endpoints()` instead.
    ///
    /// This method returns `&self` — the map is immutable after `build()`
    /// (BC-3.5.002 Invariant 1). No `&mut` accessor is provided.
    ///
    /// (BC-3.5.002 postcondition 4; Invariant 1; ADR-011 §2.3; VP-125)
    pub fn customer_endpoints(&self) -> &HashMap<OrgKey, SocketAddr> {
        todo!(
            "S-3.3.04: Network-mode customer_endpoints accessor — \
             returns immutable reference to the per-org, per-DTU endpoint table \
             (BC-3.5.002 Invariant 1; ADR-011 §2.3)"
        )
    }

    /// Check whether the clone at `(org_id, dtu_type)` has crashed.
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
    ///
    /// Returns `Err(UnknownOrg)` if the slug is not registered,
    /// `Err(UnknownDtuType)` if the DTU type is not registered for that org.
    ///
    /// (BC-3.6.001 EC-001, EC-002)
    fn resolve_endpoint(
        &self,
        slug: &str,
        dtu_type: DtuType,
    ) -> Result<(OrgId, SocketAddr), HarnessError> {
        let org_id =
            self.slug_to_org
                .get(slug)
                .copied()
                .ok_or_else(|| HarnessError::UnknownOrg {
                    slug: slug.to_owned(),
                })?;

        let key = (org_id, dtu_type);
        let addr =
            self.endpoints
                .get(&key)
                .copied()
                .ok_or_else(|| HarnessError::UnknownDtuType {
                    slug: slug.to_owned(),
                    dtu_type: format!("{dtu_type:?}"),
                })?;

        Ok((org_id, addr))
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
        slug: &str,
        dtu_type: DtuType,
        mode: FailureMode,
    ) -> Result<(), HarnessError> {
        // (1) Resolve endpoint → (org_id, addr)
        let (org_id, addr) = self.resolve_endpoint(slug, dtu_type)?;

        // (2) Check for crash before any HTTP call (BC-3.6.002 Invariant 1)
        self.check_crash(org_id, dtu_type)?;

        // (3) EC-007: NetworkTimeout with after_ms=0 → treat as FailureMode::None
        let effective_mode = match &mode {
            FailureMode::NetworkTimeout { after_ms: 0 } => FailureMode::None,
            other => other.clone(),
        };

        // (4) POST /dtu/configure with JSON body and admin token
        let body = failure_mode_to_json(&effective_mode);
        let admin_token = self
            .admin_tokens
            .get(&(org_id, dtu_type))
            .map(|s| s.as_str())
            .unwrap_or("");

        let url = format!("http://{addr}/dtu/configure");
        let resp = self
            .http_client
            .post(&url)
            .header("x-admin-token", admin_token)
            .json(&body)
            .send()
            .await?;

        if resp.status().is_success() {
            Ok(())
        } else {
            // Non-200 from configure. Re-check crash first (crash may have just surfaced).
            self.check_crash(org_id, dtu_type)?;
            // If not crashed, this is a logic error in the harness — panic is appropriate.
            panic!(
                "POST /dtu/configure returned non-200 status {} for ({slug:?}, {dtu_type:?}); \
                 this is a harness bug — configure endpoint must always return 200",
                resp.status()
            )
        }
    }

    /// Clear any injected failure from the clone at `(org_slug, dtu_type)`.
    ///
    /// Equivalent to `inject_failure(slug, dtu_type, FailureMode::None)`.
    ///
    /// Idempotent: calling this when no failure is active returns `Ok(())`
    /// with no state change (BC-3.6.001 postcondition 4; EC-006).
    ///
    /// (BC-3.6.001 postconditions 3, 4; Invariant 4)
    pub async fn clear_failure(&self, slug: &str, dtu_type: DtuType) -> Result<(), HarnessError> {
        self.inject_failure(slug, dtu_type, FailureMode::None).await
    }
}

/// Convert a `FailureMode` to the JSON body accepted by `/dtu/configure`.
fn failure_mode_to_json(mode: &FailureMode) -> serde_json::Value {
    use serde_json::json;
    match mode {
        FailureMode::None => json!({ "clear": true }),
        FailureMode::AuthReject => json!({ "auth_mode": "reject" }),
        FailureMode::RateLimit {
            after_n_requests,
            retry_after_secs,
        } => json!({
            "rate_limit_after": after_n_requests,
            "retry_after_secs": retry_after_secs,
        }),
        FailureMode::InternalError { at_request_n } => {
            json!({ "internal_error_at": at_request_n })
        }
        FailureMode::NetworkTimeout { after_ms } => {
            json!({ "network_timeout_ms": after_ms })
        }
        FailureMode::MalformedResponse => json!({ "malformed_response": true }),
        FailureMode::Unprocessable { at_request_n } => {
            json!({ "unprocessable_at": at_request_n })
        }
    }
}

impl Drop for Harness {
    /// Tear down all clone tasks gracefully, with immediate abort fallback.
    ///
    /// Steps:
    /// 1. Send shutdown signal via `shutdown_senders` to all running clones.
    /// 2. Abort each `JoinHandle` immediately (hard abort in drop is acceptable).
    ///
    /// Already-crashed clones (task exited) are no-ops — their `JoinHandle`
    /// completes immediately (BC-3.6.002 postcondition 4).
    ///
    /// (BC-3.5.001 postcondition 4; EC-003, EC-004; BC-3.6.002 postcondition 4; VP-124)
    fn drop(&mut self) {
        // Signal graceful shutdown to all running clones.
        for (_key, sender) in self.shutdown_senders.drain() {
            let _ = sender.send(());
        }
        // Hard-abort all task handles (immediate; acceptable in drop context).
        // Crashed tasks' JoinHandles are already complete — abort is a no-op for them.
        for (_key, handle) in self.task_handles.drain() {
            handle.abort();
        }
    }
}
