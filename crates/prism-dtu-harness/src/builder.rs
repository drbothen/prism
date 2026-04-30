//! `HarnessBuilder` — accumulates harness configuration before `build()`.
//!
//! `HarnessBuilder` is a pure-core builder — it accumulates `CustomerSpec`
//! values and the `IsolationMode` with no I/O until `.build().await` is called.
//!
//! # `build()` semantics (effectful-shell)
//!
//! `build()` dispatches on `IsolationMode`:
//!
//! ## `IsolationMode::Logical` (S-3.3.03)
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
//! ## `IsolationMode::Network` (S-3.3.04)
//!
//! 1. **Pre-allocation phase** — call `allocate_network_listeners` to bind all
//!    `TcpListener`s simultaneously before any `start_on` (D-058 / BC-3.5.002
//!    Invariant 2). On bind failure: `Err(HarnessError::NetworkPortAllocation { .. })`.
//!    No partial harness is returned.
//!
//! 2. **Startup phase** — spawn all clone tasks concurrently via `tokio::join!`.
//!    The entire join must complete within 5s (BC-3.5.002 postcondition 5).
//!
//! 3. **Harness construction** — populate both `endpoints` and `customer_endpoints`
//!    (BC-3.5.002 Invariant 1); return `Ok(Harness { .. })`.
//!
//! # Architecture Anchors
//!
//! - ADR-011 §2.2 — Logical mode in-process org-keyed routing
//! - ADR-011 §2.3 — Network mode: `customer_endpoints` table
//! - ADR-011 §2.5 — Port allocation: bind all listeners simultaneously before first `start_on`
//! - D-058          — 200ms budget (Logical) / no retry on EADDRINUSE (both modes)
//! - BC-3.5.001 preconditions 2-3; postconditions 1, 5; EC-003, EC-005
//! - BC-3.5.002 preconditions 1-4; postconditions 4, 5; Invariants 1-2; EC-004

use std::collections::HashMap;

use crate::clone_server::start_clone;
use crate::crash_monitor::crash_channel;
use crate::error::HarnessError;
use crate::harness::Harness;
use crate::types::{CustomerSpec, DtuType, IsolationMode, OrgKey};
use prism_core::ids::OrgId;
use prism_core::tenant::OrgSlug;

/// Builder for constructing a [`Harness`].
///
/// Created via `Harness::builder()` or `HarnessBuilder::new()`.
///
/// All methods except `build()` are synchronous and return `Self` for
/// chaining. `build()` is `async` and consumes the builder.
///
/// (Story S-3.3.03, Task 4; BC-3.5.001 precondition 2)
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
    /// - `IsolationMode::Logical` (default): in-process org-keyed routing (S-3.3.03).
    /// - `IsolationMode::Network`: per-port real HTTP routing (S-3.3.04 / BC-3.5.002).
    ///   In Network mode, `build()` pre-allocates all TCP listeners simultaneously
    ///   before any `start_on` call (D-058 pre-allocate rule; no EADDRINUSE retry).
    ///
    /// (ADR-011 §2.1; BC-3.5.001 precondition 1; BC-3.5.002 precondition 1)
    pub fn isolation(mut self, mode: IsolationMode) -> Self {
        self.isolation = mode;
        self
    }

    /// Register a customer by org slug using default `CustomerSpec` settings.
    ///
    /// The `org_id` is synthesized as a new UUID v7 derived from the registration
    /// call. Tests that need a specific `OrgId` should use `with_customer_overrides`
    /// to set `spec.org_id`.
    ///
    /// (BC-3.5.001 precondition 2; Story S-3.3.03 Task 4)
    pub fn with_customer(mut self, slug: &str) -> Self {
        let org_id = OrgId::new();
        let org_slug = OrgSlug::new(slug);
        let spec = CustomerSpec::new(org_id, org_slug);
        self.customers.push(spec);
        self
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
    pub fn with_customer_overrides(
        mut self,
        slug: &str,
        f: impl FnOnce(&mut CustomerSpec),
    ) -> Self {
        let org_id = OrgId::new();
        let org_slug = OrgSlug::new(slug);
        let mut spec = CustomerSpec::new(org_id, org_slug);
        f(&mut spec);
        self.customers.push(spec);
        self
    }

    /// Consume the builder and start the harness.
    ///
    /// # Failure modes
    ///
    /// - `HarnessError::PortConflict`          — Logical: a clone could not bind its TCP port.
    /// - `HarnessError::StartupTimeout`         — parallel startup exceeded budget (D-058).
    /// - `HarnessError::PortExhausted`          — OS could not provide ephemeral ports.
    /// - `HarnessError::NetworkPortAllocation`  — Network: simultaneous bind failed.
    ///
    /// (BC-3.5.001 precondition 3; postconditions 1, 5; EC-003, EC-005)
    /// (BC-3.5.002 preconditions 1, 4; postconditions 4, 5; EC-004)
    #[allow(clippy::expect_used)]
    pub async fn build(self) -> Result<Harness, HarnessError> {
        // Dispatch on isolation mode.
        // NOTE: `IsolationMode` is `#[non_exhaustive]` for downstream crates, but in
        // this (defining) crate Rust knows all variants — use `==` rather than a `match`
        // to avoid triggering `unreachable_patterns` when all variants are listed.
        if self.isolation == IsolationMode::Network {
            return build_network(self).await;
        }
        // IsolationMode::Logical (and any future variants not yet handled) fall through
        // to the Logical build path below.

        // Phase 1: collect all (OrgId, DtuType, slug, seed, bind_override, startup_delay_ms) tuples
        // and pre-bind one TCP listener per tuple simultaneously.
        //
        // D-058 pre-allocate rule: all listeners bound before any spawn.
        #[allow(clippy::type_complexity)]
        let mut bind_targets: Vec<(
            OrgId,
            DtuType,
            String,
            u64,
            Option<std::net::SocketAddr>,
            Option<u64>,
        )> = Vec::new();
        for spec in &self.customers {
            let slug = spec.org_slug.as_str().to_owned();
            for &dtu_type in &spec.dtu_types {
                bind_targets.push((
                    spec.org_id,
                    dtu_type,
                    slug.clone(),
                    spec.seed,
                    spec.bind_override,
                    spec.startup_delay_ms,
                ));
            }
        }

        // Bind all listeners up-front (EADDRINUSE → PortConflict).
        let mut bound: Vec<(
            OrgId,
            DtuType,
            String,
            u64,
            Option<u64>,
            tokio::net::TcpListener,
        )> = Vec::with_capacity(bind_targets.len());
        for (org_id, dtu_type, slug, seed, bind_override, startup_delay_ms) in bind_targets {
            let bind_addr = bind_override
                .map(|a| a.to_string())
                .unwrap_or_else(|| "127.0.0.1:0".to_owned());
            match tokio::net::TcpListener::bind(&bind_addr).await {
                Ok(listener) => {
                    bound.push((org_id, dtu_type, slug, seed, startup_delay_ms, listener))
                }
                Err(_) => {
                    return Err(HarnessError::PortConflict {
                        org: org_id,
                        dtu: dtu_type,
                    });
                }
            }
        }

        // Phase 2: spawn clone tasks in parallel within a 200ms wall-clock budget.
        //
        // We build all the per-clone data simultaneously, then collect into maps.
        let mut startup_futures: Vec<_> = Vec::with_capacity(bound.len());

        // Pre-create all shutdown senders and crash channels before spawning.
        let mut shutdown_senders: HashMap<OrgKey, tokio::sync::broadcast::Sender<()>> =
            HashMap::new();
        let mut crash_channels: HashMap<OrgKey, tokio::sync::watch::Receiver<Option<String>>> =
            HashMap::new();
        let mut crash_senders: HashMap<OrgKey, tokio::sync::watch::Sender<Option<String>>> =
            HashMap::new();

        for (org_id, dtu_type, ref _slug, ref _seed, ref _startup_delay_ms, ref _listener) in &bound
        {
            let key = (*org_id, *dtu_type);
            let (shutdown_tx, _) = tokio::sync::broadcast::channel::<()>(1);
            let (crash_tx, crash_rx) = crash_channel();
            shutdown_senders.insert(key, shutdown_tx);
            crash_channels.insert(key, crash_rx);
            crash_senders.insert(key, crash_tx);
        }

        // Build startup futures — each starts one clone.
        for (org_id, dtu_type, slug, seed, startup_delay_ms, listener) in bound {
            let key = (org_id, dtu_type);
            let shutdown_tx = shutdown_senders
                .get(&key)
                .expect("shutdown_tx must exist")
                .clone();
            let crash_tx = crash_senders.remove(&key).expect("crash_tx must exist");
            let shutdown_rx = shutdown_tx.subscribe();

            // Test hook: artificial startup delay (BC-3.5.001 EC-005 test path)
            let startup_delay = startup_delay_ms;
            startup_futures.push(async move {
                if let Some(delay) = startup_delay {
                    tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                }
                start_clone(listener, slug, seed, dtu_type, shutdown_rx, crash_tx).await
            });
        }

        // Run all startup futures in parallel under a 200ms timeout.
        let start_results = tokio::time::timeout(
            std::time::Duration::from_millis(200),
            start_all(startup_futures),
        )
        .await;

        match start_results {
            Err(_elapsed) => {
                // Timeout: abort all tasks (shutdown_senders still owns tx, drop aborts).
                Err(HarnessError::StartupTimeout)
            }
            Ok(started_clones) => {
                // Phase 3: populate Harness fields.
                let mut endpoints: HashMap<OrgKey, std::net::SocketAddr> = HashMap::new();
                let mut task_handles: HashMap<OrgKey, tokio::task::JoinHandle<()>> = HashMap::new();
                let mut admin_tokens: HashMap<OrgKey, String> = HashMap::new();
                // Rebuild slug→OrgId map from customers
                let mut slug_to_org: HashMap<String, OrgId> = HashMap::new();

                // Re-derive key order from customers (same order as startup_futures)
                let mut key_order: Vec<OrgKey> = Vec::new();
                for spec in &self.customers {
                    let slug = spec.org_slug.as_str().to_owned();
                    slug_to_org.insert(slug.clone(), spec.org_id);
                    for &dtu_type in &spec.dtu_types {
                        key_order.push((spec.org_id, dtu_type));
                    }
                }

                for (key, started) in key_order.into_iter().zip(started_clones) {
                    endpoints.insert(key, started.addr);
                    task_handles.insert(key, started.handle);
                    admin_tokens.insert(key, started.admin_token);
                }

                let http_client = reqwest::Client::new();

                Ok(Harness {
                    endpoints,
                    crash_channels,
                    shutdown_senders,
                    task_handles,
                    admin_tokens,
                    http_client,
                    slug_to_org,
                    // Logical mode: customer_endpoints is not used — Network mode populates it.
                    customer_endpoints: HashMap::new(),
                })
            }
        }
    }
}

/// Start all clone futures concurrently and collect results.
#[allow(clippy::expect_used)]
async fn start_all(
    futures: Vec<
        impl std::future::Future<Output = crate::clone_server::StartedClone> + Send + 'static,
    >,
) -> Vec<crate::clone_server::StartedClone> {
    let mut results = Vec::with_capacity(futures.len());
    // Spawn all futures concurrently; collect JoinHandles then await them.
    let handles: Vec<_> = futures.into_iter().map(tokio::spawn).collect();

    for handle in handles {
        results.push(handle.await.expect("clone startup task panicked"));
    }
    results
}

impl Default for HarnessBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Network-mode build path (S-3.3.04 / BC-3.5.002)
// ---------------------------------------------------------------------------

/// Pre-allocate one `TcpListener` per `(OrgKey)` simultaneously, returning
/// `(OrgKey, listener_socket_addr, TcpListener)` triples.
///
/// All listeners are bound before any is handed to a clone's `start_on` call.
/// This eliminates the bind-drop-rebind race window entirely (ADR-011 §2.5;
/// D-058; BC-3.5.002 Invariant 2).
///
/// On any bind failure, all previously-allocated listeners are dropped (their
/// OS ports are released) and `Err(HarnessError::NetworkPortAllocation { .. })`
/// is returned. No partial allocation is retained.
///
/// (BC-3.5.002 precondition 4; EC-004; ADR-011 §2.5; D-058)
// S-3.3.04 stub: called by build_network (also a stub). Both bodies are todo!() until
// the implementer phase of S-3.3.04. Suppress dead_code lint for the stub pair.
#[allow(dead_code)]
fn allocate_network_listeners(
    keys: &[OrgKey],
) -> Result<HashMap<OrgKey, (std::net::SocketAddr, std::net::TcpListener)>, HarnessError> {
    todo!(
        "S-3.3.04: allocate_network_listeners — \
         bind {} TcpListeners simultaneously on 127.0.0.1:0; \
         return HashMap<OrgKey, (SocketAddr, TcpListener)>; \
         on any failure drop all listeners and return \
         Err(HarnessError::NetworkPortAllocation {{ source: io::Error }}). \
         No retry loop (D-058). \
         (BC-3.5.002 Invariant 2; ADR-011 §2.5)",
        keys.len()
    )
}

/// Network-mode `build()` dispatch — called when `IsolationMode::Network` is set.
///
/// Three phases:
///
/// 1. Collect all `(OrgKey, slug, seed)` tuples from `builder.customers`.
/// 2. Call `allocate_network_listeners` to bind all listeners simultaneously
///    (D-058 pre-allocate rule; BC-3.5.002 Invariant 2).
/// 3. Spawn all clone tasks in parallel via `tokio::join!` within a 5s timeout
///    (BC-3.5.002 postcondition 5). Populate both `endpoints` and
///    `customer_endpoints` atomically; return `Ok(Harness { .. })`.
///
/// On any failure: drop all pre-allocated listeners, return `Err`. No partial
/// `Harness` is ever returned (BC-3.5.002 EC-003, EC-004).
///
/// (BC-3.5.002 preconditions 1, 4; postconditions 4, 5, 6; Invariants 1, 2;
///  ADR-011 §2.3, §2.5; D-058)
async fn build_network(_builder: HarnessBuilder) -> Result<Harness, HarnessError> {
    todo!(
        "S-3.3.04: build_network — \
         (1) collect (OrgKey, slug, seed) tuples from builder.customers; \
         (2) call allocate_network_listeners to bind all listeners simultaneously \
             (no EADDRINUSE retry, D-058); \
         (3) spawn all clone tasks in parallel via tokio::join! within 5s timeout \
             (BC-3.5.002 postcondition 5); \
         (4) populate endpoints and customer_endpoints atomically; \
         (5) return Ok(Harness {{ customer_endpoints, endpoints, .. }}). \
         On any failure: drop all listeners, return Err — no partial Harness. \
         (BC-3.5.002 preconditions 1, 4; postconditions 4, 5; Invariants 1, 2)"
    )
}
