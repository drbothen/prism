//! `HarnessBuilder` — accumulates harness configuration before `build()`.
//!
//! `HarnessBuilder` is a pure-core builder — it accumulates `CustomerSpec`
//! values and the `IsolationMode` with no I/O until `.build().await` is called.
//!
//! # `build()` semantics (effectful-shell)
//!
//! `build()` performs three phases:
//!
//! 1. **Bind phase** — allocate one `TcpListener` per `(OrgId, DtuType)` pair
//!    simultaneously, before any clone task is spawned (D-058 pre-allocate rule).
//!    If any bind fails, return `Err(HarnessError::PortConflict { .. })`.
//!
//! 2. **Startup phase** — spawn all clone tasks in parallel via `tokio::join!`
//!    (D-058 parallel startup rule). The entire join must complete within 200ms;
//!    if it times out, abort all partially-started tasks and return
//!    `Err(HarnessError::StartupTimeout)`.
//!
//! 3. **Harness construction** — populate the immutable `endpoints: HashMap<OrgKey, SocketAddr>`
//!    and per-clone `crash_channels` / `shutdown_senders` / `task_handles` maps;
//!    return `Ok(Harness { .. })`.
//!
//! # Architecture Anchors
//!
//! - ADR-011 §2.2 — Logical mode in-process org-keyed routing
//! - ADR-011 §2.5 — Port allocation: bind all listeners simultaneously before first `start_on`
//! - D-058          — 200ms budget locked decision; no retry on EADDRINUSE
//! - BC-3.5.001 preconditions 2-3; postconditions 1, 5; EC-003, EC-005

use crate::error::HarnessError;
use crate::harness::Harness;
use crate::types::{CustomerSpec, IsolationMode};

/// Builder for constructing a [`Harness`].
///
/// Created via `Harness::builder()` or `HarnessBuilder::new()`.
///
/// All methods except `build()` are synchronous and return `&mut Self` for
/// chaining. `build()` is `async` and consumes the builder.
///
/// (Story S-3.3.03, Task 4; BC-3.5.001 precondition 2)
// S-3.3.03 stub: fields are written by builder methods but read only in build().
// `customers` is unused until build() is implemented; suppress dead_code for stub phase.
#[allow(dead_code)]
pub struct HarnessBuilder {
    /// The isolation strategy for this harness.
    ///
    /// Defaults to `IsolationMode::Logical` — the mode implemented by S-3.3.03.
    pub(crate) isolation: IsolationMode,

    /// Registered customers in insertion order.
    ///
    /// Insertion order is preserved so that `build()` binds ports deterministically
    /// across runs (given the same OS state), making debugging easier.
    pub(crate) customers: Vec<CustomerSpec>,
}

impl HarnessBuilder {
    /// Create a new builder with `IsolationMode::Logical` and no customers.
    pub fn new() -> Self {
        Self {
            isolation: IsolationMode::Logical,
            customers: Vec::new(),
        }
    }

    /// Override the isolation mode.
    ///
    /// Calling this with `IsolationMode::Network` before S-3.3.05 lands will
    /// cause `build()` to return an error — the network-mode backend is not yet
    /// implemented.
    ///
    /// (ADR-011 §2.1; BC-3.5.001 precondition 1)
    pub fn isolation(mut self, mode: IsolationMode) -> Self {
        self.isolation = mode;
        self
    }

    /// Register a customer by org slug using default `CustomerSpec` settings.
    ///
    /// The `org_id` is synthesized as a deterministic UUID v7 derived from the
    /// slug string. Tests that need a specific `OrgId` should use
    /// `with_customer_overrides` to set `spec.org_id`.
    ///
    /// (BC-3.5.001 precondition 2; Story S-3.3.03 Task 4)
    pub fn with_customer(self, _slug: &str) -> Self {
        todo!(
            "S-3.3.03 implementation: look up slug in OrgRegistry (or synthesize OrgId), \
             construct CustomerSpec::new(org_id, OrgSlug::new(slug)), push to self.customers. \
             Return self."
        )
    }

    /// Register a customer and apply overrides via a closure.
    ///
    /// ```ignore
    /// builder.with_customer_overrides("acme-corp", |spec| {
    ///     spec.dtu_types = vec![DtuType::Claroty];
    ///     spec.seed = 99;
    /// });
    /// ```
    ///
    /// (Story S-3.3.03 Task 4; BC-3.5.001 precondition 2)
    pub fn with_customer_overrides(self, _slug: &str, _f: impl FnOnce(&mut CustomerSpec)) -> Self {
        todo!(
            "S-3.3.03 implementation: construct default CustomerSpec for slug, apply closure f, \
             push to self.customers. Return self."
        )
    }

    /// Consume the builder and start the harness.
    ///
    /// # Failure modes
    ///
    /// - `HarnessError::PortConflict` — a clone could not bind its TCP port.
    /// - `HarnessError::StartupTimeout` — parallel startup exceeded 200ms (D-058).
    /// - `HarnessError::PortExhausted` — OS could not provide ephemeral ports.
    ///
    /// (BC-3.5.001 precondition 3; postconditions 1, 5; EC-003, EC-005)
    pub async fn build(self) -> Result<Harness, HarnessError> {
        todo!(
            "S-3.3.03 implementation: \
             (1) validate isolation == Logical (else error); \
             (2) for each (org, dtu_type) bind a TcpListener on 127.0.0.1:0 — all listeners \
                 allocated before any start_on call (D-058 pre-allocate); \
             (3) spawn clone tasks in parallel via tokio::join! wrapped in \
                 tokio::time::timeout(Duration::from_millis(200), ...); \
             (4) if timeout: abort all partially-started tasks, return StartupTimeout; \
             (5) populate Harness fields and return Ok(harness). \
             See ADR-011 §2.2, §2.5; BC-3.5.001 postcondition 5."
        )
    }
}

impl Default for HarnessBuilder {
    fn default() -> Self {
        Self::new()
    }
}
