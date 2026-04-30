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
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

use crate::clone_server::{dtu_configure_pub, start_clone};
use crate::clones::crowdstrike::{start_crowdstrike_clone, start_crowdstrike_clone_network};
use crate::crash_monitor::crash_channel;
use crate::error::HarnessError;
use crate::harness::Harness;
use crate::types::{CustomerSpec, DtuType, IsolationMode, OrgKey};
use prism_core::ids::OrgId;
use prism_core::tenant::OrgSlug;
use prism_dtu_common::FailureMode;

/// Builder for constructing a [`Harness`].
///
/// Created via `Harness::builder()` or `HarnessBuilder::new()`.
///
/// All methods except `build()` are synchronous and return `Self` for
/// chaining. `build()` is `async` and consumes the builder.
///
/// # Example
///
/// ```ignore
/// let harness = HarnessBuilder::new()
///     .isolation(IsolationMode::Logical)
///     .with_customer("alpha")
///     .with_customer_overrides("alpha", |s| { s.scale = Some(2.0); s.seed_override = Some(42); })
///     .with_failure("alpha", DtuType::Claroty, FailureMode::AuthReject)
///     .build()
///     .await?;
/// ```
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

    /// Configurable Network-mode build timeout. `None` = use default 5s.
    ///
    /// (BC-3.5.002 postcondition 5; ADR-011 §2.5; AC-008)
    pub(crate) network_bind_timeout: Option<std::time::Duration>,

    /// Test hook: lifecycle counter injected into each Network-mode listener task.
    ///
    /// Each spawned task increments the counter on start and decrements on clean shutdown.
    /// `None` means no counter is wired (the default).
    ///
    /// (BC-3.5.002 postcondition 6; AC-006)
    pub(crate) task_lifecycle_counter: Option<std::sync::Arc<std::sync::atomic::AtomicUsize>>,

    /// Deferred `with_failure` calls for slugs not yet registered at call time.
    ///
    /// Each entry `(slug, dtu_type, mode)` is resolved during `build()`. If any
    /// slug is still unresolved after all customers are processed, `build()` returns
    /// `Err(HarnessError::UnknownOrg { slug })` (BC-3.6.001 EC-001; AC-005).
    pub(crate) pending_failures: Vec<(String, DtuType, FailureMode)>,
}

impl HarnessBuilder {
    /// Create a new builder with `IsolationMode::Logical` and no customers.
    pub fn new() -> Self {
        Self {
            isolation: IsolationMode::Logical,
            customers: Vec::new(),
            network_bind_timeout: None,
            task_lifecycle_counter: None,
            pending_failures: Vec::new(),
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
    /// If a `CustomerSpec` for `slug` already exists (registered via a prior
    /// `with_customer` or `with_customer_overrides` call), the closure is applied
    /// to that existing spec **in place** — the original `OrgId` is preserved and no
    /// duplicate spec is inserted (EC-003 last-write-wins; AC-002).
    ///
    /// If no spec exists for `slug`, a new spec is created with a fresh `OrgId` and
    /// the closure is applied to it before insertion (backward-compatible with
    /// calling `with_customer_overrides` without a preceding `with_customer`).
    ///
    /// Multiple calls for the same slug apply closures in call order; the last write
    /// to an overlapping field wins (EC-003).
    ///
    /// ```ignore
    /// builder
    ///     .with_customer("acme-corp")
    ///     .with_customer_overrides("acme-corp", |spec| {
    ///         spec.dtu_types = vec![DtuType::Claroty];
    ///         spec.seed_override = Some(99);
    ///     });
    /// ```
    ///
    /// (Story S-3.3.05 Task 2; BC-3.5.001 precondition 2; AC-002; EC-003)
    pub fn with_customer_overrides(
        mut self,
        slug: &str,
        f: impl FnOnce(&mut CustomerSpec),
    ) -> Self {
        // Look up an existing spec for this slug by `org_slug`.
        if let Some(existing) = self
            .customers
            .iter_mut()
            .find(|s| s.org_slug.as_str() == slug)
        {
            // Mutate the existing spec in place — preserves OrgId, no duplicate insertion.
            f(existing);
        } else {
            // No existing spec for this slug — create a new one (backward-compatible path).
            let org_id = OrgId::new();
            let org_slug = OrgSlug::new(slug);
            let mut spec = CustomerSpec::new(org_id, org_slug);
            f(&mut spec);
            self.customers.push(spec);
        }
        self
    }

    /// Inject a failure mode into a specific `(slug, dtu_type)` clone at build time.
    ///
    /// The failure is applied during `build()` before the `Harness` is returned to
    /// the caller. The first request to the target clone after `build()` returns will
    /// observe the injected mode without requiring a separate `Harness::inject_failure`
    /// call (BC-3.6.001 postcondition 1; AC-004).
    ///
    /// Passing `FailureMode::None` clears any previously set `initial_failure` on the
    /// matching spec (EC-002; BC-3.6.001 Invariant 4).
    ///
    /// # Error deferral
    ///
    /// This method is infallible at call time (AC-005). If `slug` does not match any
    /// spec registered by a prior `with_customer` or `with_customer_overrides` call,
    /// the entry is recorded as a deferred pending failure. `build()` checks all
    /// deferred entries after customer resolution and returns
    /// `Err(HarnessError::UnknownOrg { slug })` for any unresolved slug (BC-3.6.001 EC-001).
    ///
    /// # Example
    ///
    /// ```ignore
    /// let harness = HarnessBuilder::new()
    ///     .isolation(IsolationMode::Logical)
    ///     .with_customer("alpha")
    ///     .with_failure("alpha", DtuType::Claroty, FailureMode::AuthReject)
    ///     .build()
    ///     .await?;
    /// ```
    ///
    /// (S-3.3.05 Task 3; BC-3.6.001 postcondition 1; AC-003, AC-004, AC-005; ADR-011 §2.7)
    pub fn with_failure(mut self, slug: &str, dtu_type: DtuType, mode: FailureMode) -> Self {
        if let Some(existing) = self
            .customers
            .iter_mut()
            .find(|s| s.org_slug.as_str() == slug)
        {
            // Slug found — set initial_failure on the existing spec.
            // FailureMode::None clears any prior injection (EC-002).
            existing.initial_failure = if matches!(mode, FailureMode::None) {
                None
            } else {
                Some(mode)
            };
        } else {
            // Slug not yet registered — defer to build() for resolution.
            self.pending_failures
                .push((slug.to_owned(), dtu_type, mode));
        }
        self
    }

    /// Override the Network-mode build timeout (default: 5 seconds per BC-3.5.002 postcondition 5).
    ///
    /// Used in tests to exercise `StartupTimeout` with short timeouts.
    ///
    /// (BC-3.5.002 postcondition 5; ADR-011 §2.5; AC-008)
    pub fn with_network_bind_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.network_bind_timeout = Some(timeout);
        self
    }

    /// Test hook: inject a lifecycle counter into each Network-mode clone task.
    ///
    /// Each spawned task increments the counter on start and decrements on clean shutdown.
    /// Useful for verifying that `drop(harness)` joins all listener tasks (AC-006).
    ///
    /// (BC-3.5.002 postcondition 6; AC-006)
    pub fn with_task_lifecycle_counter(
        mut self,
        counter: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    ) -> Self {
        self.task_lifecycle_counter = Some(counter);
        self
    }

    /// Consume the builder and start the harness.
    ///
    /// # Failure modes
    ///
    /// - `HarnessError::UnknownOrg`             — a `with_failure` slug was not registered.
    /// - `HarnessError::PortConflict`          — Logical: a clone could not bind its TCP port.
    /// - `HarnessError::StartupTimeout`         — parallel startup exceeded budget (D-058).
    /// - `HarnessError::PortExhausted`          — OS could not provide ephemeral ports.
    /// - `HarnessError::NetworkPortAllocation`  — Network: simultaneous bind failed.
    ///
    /// (BC-3.5.001 precondition 3; postconditions 1, 5; EC-003, EC-005)
    /// (BC-3.5.002 preconditions 1, 4; postconditions 4, 5; EC-004)
    /// (BC-3.6.001 postcondition 1; EC-001)
    #[allow(clippy::expect_used)]
    pub async fn build(self) -> Result<Harness, HarnessError> {
        // Pre-injection check: resolve deferred pending_failures against registered customers.
        // Any slug that was passed to with_failure() but was never registered via with_customer()
        // or with_customer_overrides() must produce Err(UnknownOrg) here (BC-3.6.001 EC-001; AC-005).
        for (slug, _dtu_type, _mode) in &self.pending_failures {
            let known = self
                .customers
                .iter()
                .any(|s| s.org_slug.as_str() == slug.as_str());
            if !known {
                return Err(HarnessError::UnknownOrg { slug: slug.clone() });
            }
        }

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
        // S-3.3.05: use seed_override.unwrap_or(seed) as the effective seed (D-059).
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
            let effective_seed = spec.seed_override.unwrap_or(spec.seed);
            for &dtu_type in &spec.dtu_types {
                bind_targets.push((
                    spec.org_id,
                    dtu_type,
                    slug.clone(),
                    effective_seed,
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
                // Dispatch: CrowdStrike uses its own full-fidelity router;
                // all other DTU types use the generic clone_server router.
                // (S-3.4.03 CONFLICT-AVOIDANCE: only this match arm is changed.)
                match dtu_type {
                    DtuType::CrowdStrike => {
                        start_crowdstrike_clone(listener, slug, seed, shutdown_rx, crash_tx).await
                    }
                    _ => start_clone(listener, slug, seed, dtu_type, shutdown_rx, crash_tx).await,
                }
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

                let harness = Harness {
                    endpoints,
                    crash_channels,
                    shutdown_senders,
                    task_handles,
                    admin_tokens,
                    http_client,
                    slug_to_org,
                    // Logical mode: customer_endpoints is not used — Network mode populates it.
                    customer_endpoints: HashMap::new(),
                };

                // Phase 4 (S-3.3.05): apply per-customer initial_failure injections.
                //
                // Each CustomerSpec with `initial_failure = Some(mode)` gets a
                // `inject_failure` call for every registered DtuType in that org.
                // This satisfies BC-3.6.001 postcondition 1 — the first request after
                // `build()` returns observes the failure without a separate inject call.
                for spec in &self.customers {
                    if let Some(ref mode) = spec.initial_failure {
                        let slug = spec.org_slug.as_str();
                        for &dtu_type in &spec.dtu_types {
                            harness.inject_failure(slug, dtu_type, mode.clone()).await?;
                        }
                    }
                }

                Ok(harness)
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

/// Pre-allocate one `TcpListener` per `(OrgKey)` simultaneously using std blocking
/// sockets, returning `(OrgKey, SocketAddr, std::net::TcpListener)` triples.
///
/// All listeners are bound before any is handed to a clone's `start_on` call.
/// This eliminates the bind-drop-rebind race window entirely (ADR-011 §2.5;
/// D-058; BC-3.5.002 Invariant 2).
///
/// On any bind failure, all previously-allocated listeners are dropped (their
/// OS ports are released) and `Err(HarnessError::NetworkPortAllocation { .. })`
/// is returned. No partial allocation is retained.
///
/// Uses `std::net::TcpListener` (synchronous bind) so all listeners are bound
/// atomically before any async work begins — no retry loop (D-058).
///
/// (BC-3.5.002 precondition 4; EC-004; ADR-011 §2.5; D-058)
fn allocate_network_listeners(
    keys: &[OrgKey],
) -> Result<Vec<(OrgKey, std::net::SocketAddr, std::net::TcpListener)>, HarnessError> {
    let mut allocated: Vec<(OrgKey, std::net::SocketAddr, std::net::TcpListener)> =
        Vec::with_capacity(keys.len());

    for &key in keys {
        match std::net::TcpListener::bind("127.0.0.1:0") {
            Ok(listener) => {
                let addr = listener
                    .local_addr()
                    .map_err(|source| HarnessError::NetworkPortAllocation { source })?;
                allocated.push((key, addr, listener));
            }
            Err(source) => {
                // Drop all previously-allocated listeners — their OS ports are released.
                drop(allocated);
                return Err(HarnessError::NetworkPortAllocation { source });
            }
        }
    }

    Ok(allocated)
}

/// Network-mode `build()` dispatch — called when `IsolationMode::Network` is set.
///
/// Three phases:
///
/// 1. Collect all `(OrgKey, slug, seed)` tuples from `builder.customers`.
/// 2. Call `allocate_network_listeners` to bind all listeners simultaneously
///    (D-058 pre-allocate rule; BC-3.5.002 Invariant 2). Synchronous std bind —
///    no EADDRINUSE retry anywhere.
/// 3. Spawn all clone tasks in parallel via `tokio::join!` within the configured
///    timeout (default 5s per BC-3.5.002 postcondition 5). Populate both
///    `endpoints` and `customer_endpoints` atomically; return `Ok(Harness { .. })`.
///
/// On any failure: drop all pre-allocated listeners, return `Err`. No partial
/// `Harness` is ever returned (BC-3.5.002 EC-003, EC-004).
///
/// (BC-3.5.002 preconditions 1, 4; postconditions 4, 5, 6; Invariants 1, 2;
///  ADR-011 §2.3, §2.5; D-058)
#[allow(clippy::expect_used)]
async fn build_network(builder: HarnessBuilder) -> Result<Harness, HarnessError> {
    // Phase 1: collect (OrgKey, slug, seed, startup_delay_ms) tuples.
    #[allow(clippy::type_complexity)]
    let mut clone_specs: Vec<(OrgKey, String, u64, Option<u64>)> = Vec::new();
    let mut slug_to_org: HashMap<String, prism_core::ids::OrgId> = HashMap::new();

    for spec in &builder.customers {
        let slug = spec.org_slug.as_str().to_owned();
        slug_to_org.insert(slug.clone(), spec.org_id);
        // S-3.3.05: use seed_override.unwrap_or(seed) as the effective seed (D-059).
        let effective_seed = spec.seed_override.unwrap_or(spec.seed);
        for &dtu_type in &spec.dtu_types {
            clone_specs.push((
                (spec.org_id, dtu_type),
                slug.clone(),
                effective_seed,
                spec.startup_delay_ms,
            ));
        }
    }

    // Phase 2: pre-allocate all TCP listeners simultaneously (synchronous std bind).
    // D-058: all binds precede all start_on calls; no EADDRINUSE retry loop.
    let keys: Vec<OrgKey> = clone_specs.iter().map(|(key, ..)| *key).collect();
    let allocated = allocate_network_listeners(&keys)?;

    // Convert std::net::TcpListener → tokio::net::TcpListener and collect addresses.
    let mut tokio_listeners: Vec<(OrgKey, std::net::SocketAddr, tokio::net::TcpListener)> =
        Vec::with_capacity(allocated.len());
    for (key, addr, std_listener) in allocated {
        std_listener
            .set_nonblocking(true)
            .map_err(|source| HarnessError::NetworkPortAllocation { source })?;
        let tokio_listener = tokio::net::TcpListener::from_std(std_listener)
            .map_err(|source| HarnessError::NetworkPortAllocation { source })?;
        tokio_listeners.push((key, addr, tokio_listener));
    }

    // Phase 3: spawn all clone tasks in parallel under the configured timeout.
    let timeout_duration = builder
        .network_bind_timeout
        .unwrap_or(std::time::Duration::from_secs(5));

    // Capture customers for Phase 5 initial_failure injection (builder is consumed below).
    let customers_for_injection: Vec<_> = builder
        .customers
        .iter()
        .filter(|s| s.initial_failure.is_some())
        .map(|s| {
            (
                s.org_slug.as_str().to_owned(),
                s.dtu_types.clone(),
                s.initial_failure.clone(),
            )
        })
        .collect();

    // Build shutdown channel and crash channels before spawning.
    let (shutdown_tx, _) = tokio::sync::broadcast::channel::<()>(16);
    let mut crash_channels: HashMap<OrgKey, tokio::sync::watch::Receiver<Option<String>>> =
        HashMap::new();
    let mut crash_senders: HashMap<OrgKey, tokio::sync::watch::Sender<Option<String>>> =
        HashMap::new();

    for &(key, ..) in &tokio_listeners {
        let (crash_tx, crash_rx) = crash_channel();
        crash_channels.insert(key, crash_rx);
        crash_senders.insert(key, crash_tx);
    }

    // Build startup futures — each converts a std listener + starts one clone.
    // Extract startup delays from clone_specs (keyed by position = same order as tokio_listeners).
    let startup_delays: HashMap<OrgKey, Option<u64>> = clone_specs
        .iter()
        .map(|(key, _, _, delay)| (*key, *delay))
        .collect();

    let task_lifecycle_counter = builder.task_lifecycle_counter.clone();

    let mut startup_futures: Vec<_> = Vec::with_capacity(tokio_listeners.len());
    for (key, _addr, listener) in tokio_listeners {
        let (org_id, dtu_type) = key;
        let slug = clone_specs
            .iter()
            .find(|(k, ..)| *k == key)
            .map(|(_, s, ..)| s.clone())
            .expect("key must exist in clone_specs");
        let seed = clone_specs
            .iter()
            .find(|(k, ..)| *k == key)
            .map(|(_, _, s, _)| *s)
            .expect("key must exist in clone_specs");
        let startup_delay = startup_delays.get(&key).copied().flatten();
        let shutdown_rx = shutdown_tx.subscribe();
        let crash_tx = crash_senders.remove(&key).expect("crash_tx must exist");
        let counter = task_lifecycle_counter.clone();

        startup_futures.push(async move {
            if let Some(delay) = startup_delay {
                tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
            }
            // Dispatch: CrowdStrike uses its own full-fidelity network router with
            // bearer-token validation for cross-org 401 detection.
            // (S-3.4.03 CONFLICT-AVOIDANCE: only this match arm is changed.)
            let started = match dtu_type {
                DtuType::CrowdStrike => {
                    start_crowdstrike_clone_network(
                        listener,
                        slug,
                        seed,
                        shutdown_rx,
                        crash_tx,
                        counter,
                    )
                    .await
                }
                _ => {
                    start_clone_network(
                        listener,
                        slug,
                        seed,
                        dtu_type,
                        shutdown_rx,
                        crash_tx,
                        counter,
                    )
                    .await
                }
            };
            (key, org_id, started)
        });
    }

    // Run all startup futures in parallel under the timeout.
    let start_results = tokio::time::timeout(timeout_duration, start_all_network(startup_futures))
        .await
        .map_err(|_| HarnessError::StartupTimeout)?;

    // Phase 4: populate Harness fields.
    let mut endpoints: HashMap<OrgKey, std::net::SocketAddr> = HashMap::new();
    let mut customer_endpoints: HashMap<OrgKey, std::net::SocketAddr> = HashMap::new();
    let mut task_handles: HashMap<OrgKey, tokio::task::JoinHandle<()>> = HashMap::new();
    let mut admin_tokens: HashMap<OrgKey, String> = HashMap::new();

    for (key, started) in start_results {
        endpoints.insert(key, started.addr);
        customer_endpoints.insert(key, started.addr);
        task_handles.insert(key, started.handle);
        admin_tokens.insert(key, started.admin_token);
    }

    // Wrap the shutdown sender in a per-key map (one shared sender, many receivers).
    let mut shutdown_senders: HashMap<OrgKey, tokio::sync::broadcast::Sender<()>> = HashMap::new();
    for &key in endpoints.keys() {
        shutdown_senders.insert(key, shutdown_tx.clone());
    }

    let http_client = reqwest::Client::new();

    let harness = Harness {
        endpoints,
        crash_channels,
        shutdown_senders,
        task_handles,
        admin_tokens,
        http_client,
        slug_to_org,
        customer_endpoints,
    };

    // Phase 5 (S-3.3.05): apply per-customer initial_failure injections before returning.
    //
    // BC-3.6.001 postcondition 1: the first request after build() returns observes the
    // injected failure without requiring a separate inject_failure call.
    for (slug, dtu_types, maybe_mode) in customers_for_injection {
        if let Some(mode) = maybe_mode {
            for dtu_type in dtu_types {
                harness
                    .inject_failure(&slug, dtu_type, mode.clone())
                    .await?;
            }
        }
    }

    Ok(harness)
}

/// Start all Network-mode clone futures concurrently and collect results.
#[allow(clippy::expect_used)]
async fn start_all_network<F, K>(futures: Vec<F>) -> Vec<(K, crate::clone_server::StartedClone)>
where
    F: std::future::Future<Output = (K, prism_core::ids::OrgId, crate::clone_server::StartedClone)>
        + Send
        + 'static,
    K: Send + 'static,
{
    let mut results = Vec::with_capacity(futures.len());
    let handles: Vec<_> = futures.into_iter().map(tokio::spawn).collect();

    for handle in handles {
        let (key, _org_id, started) = handle.await.expect("clone startup task panicked");
        results.push((key, started));
    }
    results
}

/// Start a Network-mode harness clone on the given pre-bound Tokio TCP listener.
///
/// The HTTP server uses axum (same as `clone_server.rs`) and validates
/// `Authorization: Bearer <admin_token>` on device-list routes when the header
/// is present. If no Authorization header is sent, the request is allowed through
/// (unauthenticated callers get data; only wrong tokens are rejected with 401).
///
/// This enables the cross-org credential routing tests (VP-126; AC-004; TV-3):
/// a request bearing OrgA's token sent to OrgB's endpoint returns 401.
///
/// (BC-3.5.002 postcondition 2; VP-126; ADR-003 Amendment §5)
#[allow(clippy::expect_used)]
async fn start_clone_network(
    listener: tokio::net::TcpListener,
    org_slug: String,
    seed: u64,
    dtu_type: crate::types::DtuType,
    shutdown_rx: tokio::sync::broadcast::Receiver<()>,
    crash_tx: tokio::sync::watch::Sender<Option<String>>,
    task_lifecycle_counter: Option<Arc<AtomicUsize>>,
) -> crate::clone_server::StartedClone {
    use crate::clone_server::{CloneState, StartedClone};

    let addr = listener
        .local_addr()
        .expect("network listener must have local addr after bind");

    let admin_token = uuid::Uuid::new_v4().to_string();
    let state = Arc::new(CloneState::new(
        org_slug,
        seed,
        dtu_type,
        admin_token.clone(),
    ));

    // Build a router with bearer-token validation middleware on device routes.
    // The middleware: if Authorization: Bearer <t> is present and t ≠ admin_token → 401.
    // If no Authorization header → allow (unauthenticated access is permitted for data reads).
    let router = build_network_router(Arc::clone(&state));
    let state_for_hook = Arc::clone(&state);

    let counter_clone = task_lifecycle_counter.clone();
    let handle = tokio::spawn(async move {
        // Increment lifecycle counter on task start.
        if let Some(ref counter) = counter_clone {
            counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }

        let server_future = run_network_server(listener, router, shutdown_rx);
        let hook_future = crate::clone_server::poll_test_hook_pub(state_for_hook, crash_tx.clone());

        tokio::select! {
            result = server_future => {
                if let Err(e) = result {
                    let cause = format!("network server error: {e}");
                    let _ = crash_tx.send(Some(cause));
                }
            }
            _ = hook_future => {}
        }

        // Decrement lifecycle counter on task exit.
        if let Some(ref counter) = counter_clone {
            counter.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        }
    });

    StartedClone {
        addr,
        handle,
        admin_token,
        state,
    }
}

/// Check bearer token: returns `Some(401 response)` if a wrong token was supplied.
/// Returns `None` if no Authorization header or if the token matches.
///
/// Policy: only reject if a Bearer token IS supplied AND it doesn't match.
/// Unauthenticated requests (no Authorization header) are allowed through —
/// this enables `fetch_network_devices` helpers to work without tokens.
///
/// (BC-3.5.002 postcondition 2; VP-126; AC-004)
fn check_bearer(
    headers: &axum::http::HeaderMap,
    admin_token: &str,
) -> Option<axum::response::Response> {
    use axum::{http::StatusCode, response::IntoResponse, Json};
    use serde_json::json;

    if let Some(auth_val) = headers.get("authorization") {
        if let Ok(auth_str) = auth_val.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                if token != admin_token {
                    return Some(
                        (
                            StatusCode::UNAUTHORIZED,
                            Json(json!({"error": "invalid bearer token"})),
                        )
                            .into_response(),
                    );
                }
            }
        }
    }
    None
}

/// Build a Network-mode axum router with bearer-token validation on device-list routes.
///
/// Bearer token validation logic (via `check_bearer`):
/// - If `Authorization: Bearer <token>` is present and `<token>` ≠ `admin_token` → HTTP 401.
/// - If no Authorization header → allow (unauthenticated reads are permitted).
/// - If `Authorization: Bearer <token>` matches → allow.
///
/// Also includes `POST /dtu/configure` (same as logical-mode) so that `inject_failure`
/// works identically in both isolation modes (S-3.3.05; BC-3.6.001 postcondition 1).
///
/// (BC-3.5.002 postcondition 2; VP-126; AC-004)
fn build_network_router(state: Arc<crate::clone_server::CloneState>) -> axum::Router {
    use axum::{
        http::HeaderMap,
        http::StatusCode,
        response::IntoResponse,
        routing::{get, post},
        Json,
    };
    use serde_json::json;

    let make_device_handler = |s: Arc<crate::clone_server::CloneState>, key: &'static str| {
        move |headers: HeaderMap| {
            let s = Arc::clone(&s);
            async move {
                if let Some(reject) = check_bearer(&headers, &s.admin_token) {
                    return reject;
                }
                crate::clone_server::handle_device_list_pub(s, key).await
            }
        }
    };

    axum::Router::new()
        .route(
            "/assets/v1/assets",
            get(make_device_handler(Arc::clone(&state), "assets")),
        )
        .route(
            "/api/v1/devices",
            get(make_device_handler(Arc::clone(&state), "devices")),
        )
        .route(
            "/devices/v2/devices",
            get(make_device_handler(Arc::clone(&state), "devices")),
        )
        .route(
            "/api/v1/events",
            get(make_device_handler(Arc::clone(&state), "items")),
        )
        .route(
            "/api/v1/items",
            get(make_device_handler(Arc::clone(&state), "items")),
        )
        .route(
            "/dtu/health",
            get(|| async { (StatusCode::OK, Json(json!({"status": "ok"}))).into_response() }),
        )
        // DTU configure: required for inject_failure / clear_failure in Network mode.
        // Reuses the same handler as the logical-mode clone server (S-3.3.05).
        .route("/dtu/configure", post(dtu_configure_pub))
        .with_state(Arc::clone(&state))
}

/// Run the Network-mode axum server until the shutdown signal fires.
async fn run_network_server(
    listener: tokio::net::TcpListener,
    router: axum::Router,
    mut shutdown_rx: tokio::sync::broadcast::Receiver<()>,
) -> Result<(), anyhow::Error> {
    axum::serve(listener, router)
        .with_graceful_shutdown(async move {
            let _ = shutdown_rx.recv().await;
        })
        .await
        .map_err(|e| anyhow::anyhow!("network axum serve error: {e}"))
}
