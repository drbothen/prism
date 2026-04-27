---
document_type: adr
adr_id: ADR-011
title: "DTU Harness Isolation Modes — Logical (In-Process) and Network (Per-Port)"
status: PROPOSED
date: 2026-04-27
wave: 3
phase: 3.A
version: "0.6"
authors: [architect]
related_decisions: [D-044, D-045, D-058]
related_adrs: [ADR-006, ADR-007, ADR-008]
anchored_capabilities: [CAP-036]
related_bcs_planned: [BC-3.5.001, BC-3.5.002, BC-3.6.001, BC-3.6.002]
subsystems_affected: [SS-01, SS-05, SS-06]
supersedes: null
superseded_by: null
traces_to: specs/architecture/ARCH-INDEX.md
inputs:
  - crates/prism-dtu-common/src/clone.rs
  - crates/prism-dtu-common/src/config.rs
  - crates/prism-dtu-common/src/layers/failure.rs
  - crates/prism-dtu-claroty/src/state.rs
  - crates/prism-dtu-claroty/src/lib.rs
  - .factory/STATE.md (D-044, D-045)
---

# ADR-011: DTU Harness Isolation Modes — Logical (In-Process) and Network (Per-Port)

## Status

PROPOSED — decision D-044 recorded ("Network isolation NOT deferred"). BCs to be
authored in subsequent Phase 3.A spec-writer dispatch. Implementation BLOCKED until
Phase 3.A converges (D-045).

---

## 1. Context

### 1.1 The Isolation Testing Gap

Wave 3 introduces multi-tenant DTU topology (ADR-006): multiple customer organizations,
each with independent `OrgId`-keyed sensor instances, running within a single Prism MCP
process. The behavioral contracts in the BC-3.2 family (cross-tenant data isolation,
credential isolation) must be verified, not merely asserted by inspection.

The current `BehavioralClone` trait (`crates/prism-dtu-common/src/clone.rs`) supports
single-tenant testing: one clone instance per sensor type, one test context. The
`start_on(addr, shutdown, tls_config)` method starts a stub server on an OS-assigned
port; the test constructs an HTTP client pointed at that port; assertions run against
HTTP responses. This model does not compose across multiple organizations because:

1. **No org-keyed routing.** The existing clone infrastructure has no concept of
   `OrgId`. `ClarotyClone::start()` starts one server serving all requests identically.
   In a multi-tenant context, three simultaneous `OrgId` values must each have their
   own clone instance with distinct state.

2. **In-process keying bugs are invisible.** If the multi-tenant DTU state store
   (post-ADR-008, `HashMap<(OrgId, String), V>`) has a keying bug — say, a query
   for `OrgId(A)` reads entries written by `OrgId(B)` due to a key-construction
   error — a logical isolation check within the same process may not catch it because
   the HashMap lookup path and the test assertion path share the same memory. A real
   HTTP boundary would force the bug to manifest as a visible response-level difference.

3. **Network-level isolation cannot be verified in-process.** If two organizations
   are served by DTU instances on different TCP ports, a request bearing `OrgId(A)`'s
   credentials that is accidentally routed to `OrgId(B)`'s port will fail at the
   authentication layer — an observable and testable failure. This class of cross-process
   routing bug is structurally undetectable in a single-process model.

Decision D-044 records that network isolation is NOT deferred to a future wave. Both
isolation modes ship in Wave 3.

### 1.2 Existing Infrastructure Reuse

The `BehavioralClone` trait already provides:
- `start_on(addr: SocketAddr, shutdown: Option<oneshot::Receiver<()>>, tls: Option<...>)`
  for starting a stub server on a specific address.
- `stop()` for hard-abort of a running server task.
- `FailureLayer` and `FailureLayerShared`
  (`crates/prism-dtu-common/src/layers/failure.rs`) for per-request failure injection.
- `StubConfig { seed, latency_ms, failure_mode, bind }` for per-clone configuration.

ADR-011 builds on this infrastructure. The harness is a new `prism-dtu-harness` crate
that orchestrates multiple `BehavioralClone` instances — one per `(OrgId, DtuType)`
pair — and exposes the org-keyed routing table to tests.

---

## 2. Decision

### 2.1 Two Isolation Modes, Both Shipped in Wave 3

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsolationMode {
    /// Single process, in-memory org-keyed routing.
    /// Each DTU clone has per-tenant state via (OrgId, DtuType) keying.
    /// Spinup cost: microseconds. Default for unit tests.
    Logical,

    /// Each DTU instance on its own TCP port.
    /// Harness builds a customer_endpoints table for real HTTP routing.
    /// Catches cross-process leakage that Logical mode cannot.
    /// Default for integration and multi-customer scale tests.
    Network,
}
```

Both modes use the same `Harness` builder API and produce the same test assertion
interface. The difference is entirely in how clone instances are started and how
the Prism MCP sensor client is configured to reach them.

### 2.2 Logical Mode — In-Process Org-Keyed Routing

In logical mode, all clone instances run as in-process tasks (Tokio tasks, not
OS processes). The harness maintains:

```rust
pub struct Harness {
    mode: IsolationMode,
    /// Map from (OrgId, DtuType) to the clone's bound SocketAddr.
    /// In Logical mode, all addrs are on 127.0.0.1 with distinct ports.
    /// The SocketAddr exists even in Logical mode — clones always start
    /// a real TCP listener to avoid requiring a mock HTTP layer.
    endpoints: HashMap<(OrgId, DtuType), SocketAddr>,
    /// Clone instances keyed by (OrgId, DtuType) for reconfiguration.
    clones: HashMap<(OrgId, DtuType), Box<dyn BehavioralClone>>,
    /// Shutdown senders for graceful teardown on Harness::drop.
    shutdown_senders: Vec<oneshot::Sender<()>>,
}
```

Logical mode uses `TcpListener::bind("127.0.0.1:0")` for each clone, then queries
the assigned port before starting the clone on that address via `start_on`. All
clones run within the test process's Tokio runtime. The `endpoints` table is
populated during `Harness::build()`.

Spinup time: microseconds per clone. A 3-org × 4-sensor harness (12 clone instances)
starts in under 50ms on a development machine. Logical mode is the default for the
99% of unit tests that do not require cross-process isolation verification.

### 2.3 Network Mode — Per-Port OS-Process Isolation

In network mode, each `(OrgId, DtuType)` pair gets its own OS-level TCP port.
The harness starts each clone as a separate Tokio task with an OS-assigned ephemeral
port, exactly as in logical mode — but the Prism MCP sensor client is configured
with the full per-org endpoint table, and tests assert on real HTTP responses across
the loopback interface.

The key behavioral difference from logical mode: the Prism MCP layer uses the
`endpoints` table to route sensor requests. A keying bug that sends an `OrgId(A)`
request to the `OrgId(B)` clone address will hit the wrong clone's authentication
middleware — the clone's admin token check (`ClarotyState::admin_token`, per
ADR-003 Amendment §5) will reject a request bearing the wrong org's credentials.
This observable HTTP 401 makes the routing bug detectable.

```rust
/// Per-org, per-DTU endpoint table.
/// Network mode: real loopback TCP. Logical mode: also real TCP, same structure.
pub type CustomerEndpoints = HashMap<(OrgId, DtuType), SocketAddr>;
```

Network mode is the default for integration tests and the cross-tenant fidelity
test battery (Section 2.6).

### 2.4 Harness Builder API

```rust
impl HarnessBuilder {
    /// Set the isolation mode for this harness instance.
    pub fn isolation(mut self, mode: IsolationMode) -> Self;

    /// Register a customer organization with its sensor configuration.
    /// GenOpts defaults to archetype from TOML or HealthyOtEnvironment if absent.
    pub fn with_customer(
        mut self,
        org_slug: &str,
        f: impl FnOnce(CustomerBuilder) -> CustomerBuilder,
    ) -> Self;

    /// Build and start all clone instances. Allocates ports, starts Tokio tasks.
    /// Returns Err if any clone fails to start within the startup timeout.
    /// All clone startup is parallelized via tokio::join! — the timeout applies
    /// to the entire harness build, not per-clone.
    pub async fn build(self) -> anyhow::Result<Harness>;
}

impl CustomerBuilder {
    /// Override archetype for a specific DTU type for this customer.
    pub fn dtu(self, dtu_type: &str) -> DtuBuilder;
}

impl DtuBuilder {
    pub fn set_archetype(mut self, archetype: Archetype) -> Self;
    pub fn set_seed(mut self, seed: u64) -> Self;
    pub fn set_scale(mut self, scale: f64) -> Self;
    pub fn set_failure_mode(mut self, mode: FailureMode) -> Self;
}

// Example usage
let harness = Harness::builder()
    .isolation(IsolationMode::Network)
    .with_customer("acme-corp", |c| {
        c.dtu("claroty").set_archetype(Archetype::HealthyOtEnvironment)
         .dtu("crowdstrike").set_archetype(Archetype::CompromisedEndpoint)
    })
    .with_customer("globex", |c| {
        c.dtu("crowdstrike").set_archetype(Archetype::AuthOutage)
    })
    .build()
    .await?;
```

### 2.5 Port Allocation and Cleanup

Port allocation follows the OS-assigned ephemeral port pattern already established
by `BehavioralClone::start()`:

```rust
// Allocate port: bind, query assigned port, drop listener, pass addr to start_on.
let listener = TcpListener::bind("127.0.0.1:0").await?;
let addr = listener.local_addr()?;
drop(listener); // Release before clone binds to avoid EADDRINUSE race
clone.start_on(addr, Some(shutdown_rx), None).await?;
```

On `Harness::drop`, all shutdown senders are consumed, triggering graceful shutdown
of each clone's Tokio task. If a clone does not shut down within 5 seconds, `stop()`
is called (hard abort). Port release is implicit: when the TCP listener closes, the
OS reclaims the ephemeral port.

There is no persistent port registry. Port assignments are ephemeral per test run.
Two parallel test runs (e.g., `cargo test -- --test-threads=4`) receive independent
port sets from the OS.

### 2.6 Crash Detection

The harness monitors each clone's Tokio task handle after startup. If a clone task
exits unexpectedly (panics, returns `Err`, or exits `Ok` before the test completes),
the harness marks that `(OrgId, DtuType)` as crashed and any subsequent request
to that endpoint fails the test with an explicit error:

```
HarnessError::CloneCrashed {
    org_id: OrgId(...),
    dtu_type: DtuType::Claroty,
    cause: "task panicked: index out of bounds",
}
```

This prevents a crashed clone from producing silent `ConnectionRefused` errors
that would be misinterpreted as test infrastructure failures.

### 2.7 Failure Injection API

Each `(OrgId, DtuType)` clone supports independent runtime failure injection via the
existing `FailureLayerShared` infrastructure
(`crates/prism-dtu-common/src/layers/failure.rs`):

```rust
impl Harness {
    /// Inject a failure mode into a specific (org, dtu) clone.
    pub fn inject_failure(
        &self,
        org_slug: &str,
        dtu_type: &str,
        mode: FailureMode,
    ) -> anyhow::Result<()>;

    /// Clear all failure injection for a specific (org, dtu) clone.
    pub fn clear_failure(&self, org_slug: &str, dtu_type: &str) -> anyhow::Result<()>;
}
```

`FailureMode` variants available (from `prism-dtu-common/src/config.rs`):
- `FailureMode::None` — no injection
- `FailureMode::RateLimit { after_n_requests }` — HTTP 429 after N requests
- `FailureMode::InternalError { after_n_requests }` — HTTP 500 after N requests
- `FailureMode::Timeout { after_n_requests, delay_ms }` — delayed response
- `FailureMode::AuthReject` — HTTP 401 on all requests
- `FailureMode::MalformedResponse` — response body is not valid JSON

The `inject_failure` path uses `POST /dtu/configure` on the clone's admin endpoint,
authenticated with `ClarotyState::admin_token` (the per-clone admin secret established
by ADR-003 Amendment §5). This is the same mechanism used by existing single-tenant
DTU tests.

### 2.8 Cross-Customer Fidelity Test (BC-3.5.002)

The harness ships a built-in test battery that exercises the isolation invariants
from BC-3.2.001 and BC-3.2.002 (ADR-006). The canonical cross-customer fidelity
test spins up three organizations:

```rust
let harness = Harness::builder()
    .isolation(IsolationMode::Network)
    .with_customer("acme-corp", |c| {
        c.dtu("claroty").set_archetype(Archetype::HealthyOtEnvironment)
         .dtu("armis").set_archetype(Archetype::HealthyOtEnvironment)
    })
    .with_customer("globex", |c| {
        c.dtu("crowdstrike").set_archetype(Archetype::CompromisedEndpoint)
    })
    .with_customer("initech", |c| {
        c.dtu("cyberint").set_archetype(Archetype::HealthyOtEnvironment)
         .dtu("crowdstrike").set_archetype(Archetype::HealthyOtEnvironment)
    })
    .build()
    .await?;
```

The fidelity test asserts:

1. **Data isolation (BC-3.5.001):** A Prism MCP query scoped to `org="acme-corp"`
   returns only records whose device IDs carry the `acme-corp` org prefix (per
   ADR-009 Section 2.5). No `globex` or `initech` prefixed IDs appear.

2. **Cross-tenant non-leakage (BC-3.5.002):** The intersection of device ID sets
   across all three organizations is empty. For all pairs `(OrgA, OrgB)` where
   `OrgA ≠ OrgB`, `devices(OrgA) ∩ devices(OrgB) = ∅`.

3. **Credential isolation (BC-3.2.002 coverage):** Injecting `FailureMode::AuthReject`
   into `acme-corp`'s Claroty clone causes `acme-corp` Claroty queries to fail with
   auth errors, while `globex` and `initech` queries to their respective sensors
   continue to succeed.

4. **No cross-org state contamination:** After Org A performs a write operation
   (e.g., tag assignment on a Claroty device), a read operation scoped to Org B
   does not see the written value in Org B's device records.

### 2.9 Crate: `prism-dtu-harness`

The harness is a new workspace crate: `crates/prism-dtu-harness`. Crate structure
(per ADR-012 `src/` convention):

```
crates/prism-dtu-harness/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── builder.rs       # HarnessBuilder, CustomerBuilder, DtuBuilder
│   ├── harness.rs       # Harness struct, endpoint table, crash detection
│   ├── isolation.rs     # IsolationMode enum, port allocation
│   └── fidelity.rs      # cross-customer fidelity test battery
└── tests/
    └── cross_tenant_isolation.rs  # integration test: 3-org fidelity run
```

Gate: `#[cfg(any(test, feature = "dtu"))]`. Never links into production binary.
Depends on: `prism-dtu-common`, `prism-core` (for `OrgId`, `OrgRegistry`), and all
four client-mode DTU crates (`prism-dtu-claroty`, `prism-dtu-armis`,
`prism-dtu-crowdstrike`, `prism-dtu-cyberint`).

---

## Rationale

The two-mode design directly addresses the gap between what can be verified in-process
and what requires a real network boundary.

**Logical mode is the correct default for unit tests.** The overwhelming majority of
DTU behavioral contract tests (the BC-2.x and BC-3.x families) assert on HTTP
response content, status codes, and pagination behavior. These tests do not require
cross-process isolation verification; they require fast, isolated, reproducible clone
instances. Logical mode spinup in microseconds per clone means a 12-clone multi-tenant
harness adds negligible overhead to `cargo test`. Making network mode the universal
default would add seconds of spinup to every unit test run — unacceptable for the
developer iteration loop.

**Network mode is required for isolation verification, not performance.** The argument
for network mode is not that it is "more realistic" in a general sense — it is that
a specific class of bug (cross-org routing errors in the Prism MCP sensor dispatch
layer) is structurally undetectable without a real HTTP boundary. When Prism MCP
dispatches a sensor request, it must select the correct `(OrgId, DtuType)` endpoint
from the `CustomerEndpoints` table. A bug in that selection logic will, in network
mode, route the request to the wrong clone's authentication middleware and produce
an observable HTTP 401. In logical mode, that same bug would either access the wrong
in-process HashMap bucket (producing a data mismatch, which the fidelity test catches)
or hit the wrong in-process clone (also detectable). Network mode provides a defense
in depth layer, not the sole detection mechanism.

**D-044 (not deferred) is justified by verification completeness.** The BC-3.2 family
(ADR-006) requires proof of cross-tenant isolation. A formal proof over the type system
(Kani, `HashMap<(OrgId, String), V>` key typing) verifies that the storage layer cannot
mix tenants. But it does not verify that the HTTP routing layer correctly selects the
right storage context. Network mode tests fill that gap at the integration level.
Deferring network mode would have left the HTTP routing layer unverified in Wave 3
despite the Wave 3 multi-tenant charter. D-044 prevents that gap from being opened.

**The crash-detection mechanism converts silent failures to loud ones.** Without crash
detection, a clone that panics during a test produces a `ConnectionRefused` error on
the next HTTP request. That error is attributed to "network instability" or "port
allocation race" by the developer, wasting debugging time. Explicit `HarnessError::CloneCrashed`
with the panic message surfaces the real cause immediately.

**Failure injection per `(OrgId, DtuType)` is required for BC-3.6.x coverage.** The
BC-3.6 family specifies resilience behavior: what happens when one customer's sensor
is degraded while another customer's sensors are healthy. Injecting `FailureMode::AuthReject`
into `acme-corp`'s Claroty clone while `globex`'s Claroty clone remains healthy is
only expressible with per-org, per-sensor failure injection granularity. A harness
that supports only per-sensor-type failure injection (affecting all orgs equally)
cannot test this scenario.

---

## 3. Threat Model

### 3.1 Port Allocation Race (EADDRINUSE)

**Threat:** Between the `drop(listener)` call and `clone.start_on(addr, ...)`, the OS
assigns the same ephemeral port to another process.

**Mitigation (updated per D-058):** The preferred approach is to pre-allocate all listeners
simultaneously — hold all `TcpListener` sockets open during `build()`, then pass each bound
address to the corresponding `start_on` call before dropping. Pre-allocating simultaneously
eliminates the race window entirely (no gap between `drop(listener)` and `clone.start_on`).
This approach was evaluated during BC authoring (Phase 3.A Open Question 1 in §8) and
selected as the implementation standard via D-058. If pre-allocation fails for any clone
(OS port exhaustion), the harness returns `HarnessError::PortConflict` with the offending
`(org, dtu)` pair — no retry loop.

### 3.2 Cross-Tenant Routing Bug Escapes Logical Mode

**Threat:** A bug in the Prism MCP sensor dispatch selects the wrong `(OrgId, DtuType)`
endpoint. In logical mode, all clones are in-process, and the bug might produce a
data-level response rather than an auth failure.

**Mitigation:** Network mode integration tests (BC-3.5.001 and BC-3.5.002) run the
canonical cross-customer fidelity test in `IsolationMode::Network`. The `tests/`
directory in `prism-dtu-harness` includes `cross_tenant_isolation.rs` as a required
integration test that CI must pass before any multi-tenant story is marked done.

### 3.3 Clone Crash During Test Produces Misleading Assertion

**Threat:** A clone task panics mid-test. The test receives a `ConnectionRefused` on
the next request and attributes it to a test logic error rather than the clone crash.

**Mitigation:** Crash detection (Section 2.6) converts this to an explicit
`HarnessError::CloneCrashed`. The harness polls task handles on every request via
a `try_recv` on the crash notification channel.

---

## 4. Migration Strategy

**Step 1 — New crate scaffold.**
Create `crates/prism-dtu-harness/` with the directory layout in Section 2.9.
Add to workspace `Cargo.toml` members list. Add `#[cfg(any(test, feature = "dtu"))]`
gate. Gate: `cargo build --workspace` clean.

**Step 2 — Logical mode implementation.**
Implement `HarnessBuilder`, `Harness`, and `IsolationMode::Logical`. Port allocation
via `TcpListener::bind("127.0.0.1:0")`. Integration with `BehavioralClone::start_on`.
Implement crash detection via Tokio `JoinHandle::try_join`. Implement
`inject_failure` via `POST /dtu/configure`.
Gate: `cargo test -p prism-dtu-harness -- logical` green.

**Step 3 — Generator integration.**
Wire `HarnessBuilder::with_customer` to call `generate()` from ADR-009 for each
`(OrgId, DtuType, Archetype, GenOpts)` triple. Populate clone state from generator
output before `start_on`.
Gate: harness starts with generator-populated data; a Claroty query returns
org-tagged device IDs.

**Step 4 — Network mode implementation.**
`IsolationMode::Network` implementation is identical to Logical at the port-allocation
level (all clones still use `127.0.0.1:0`). The difference is in how the Prism MCP
test client is configured: network mode provides the full `CustomerEndpoints` table
to the client constructor so that requests are routed by `(OrgId, DtuType)` lookup,
not by a shared in-process reference.
Gate: `cargo test -p prism-dtu-harness -- network` green.

**Step 5 — Cross-customer fidelity test.**
Implement `tests/cross_tenant_isolation.rs` using the 3-org scenario from Section 2.8.
This test runs in `IsolationMode::Network` and must pass before any Wave 3 multi-tenant
story is closed.
Gate: test passes under `cargo test -p prism-dtu-harness`.

---

## 5. Alternatives Considered

| Option | Description | Decision |
|--------|-------------|----------|
| **Logical mode only** | All multi-tenant tests in-process; no per-port isolation | Rejected: D-044. HTTP routing bugs are structurally undetectable without a network boundary. |
| **Network mode only** | All tests use real TCP ports; no in-process mode | Rejected: spinup overhead makes unit test iteration impractical. Logical mode's microsecond spinup is essential for TDD workflows. |
| **Docker Compose per org** | Each org's DTU fleet in a separate Docker Compose service | Rejected for Wave 3: Docker Compose adds CI environment dependencies, significantly increases test startup time (seconds vs microseconds), and requires Docker daemon availability in all dev environments. Loopback TCP achieves the same routing isolation without infrastructure dependencies. Deferred to Wave 4 if production-environment simulation is required. |
| **Shared port, path-based routing** | Single clone port; org-keyed by URL path prefix | Rejected: path-based routing within a single clone instance re-centralizes the routing logic into the clone, defeating the purpose of per-org isolation. The `(OrgId, DtuType) → SocketAddr` table model mirrors the real deployment topology more faithfully. |
| **Mock HTTP layer, no real TCP** | Use `tower-test` or `axum::extract::State` mocking | Rejected: mock HTTP layers cannot catch routing-level bugs in the Prism MCP sensor client. The client's endpoint selection code must run against real TCP sockets to be testable. |

---

## 6. Consequences

### Positive

- Logical mode enables fast TDD cycles: a 12-clone harness starts in under 50ms.
- Network mode makes HTTP routing bugs visible at the integration test level.
- Per-`(OrgId, DtuType)` failure injection enables BC-3.6.x resilience scenarios.
- Crash detection converts silent `ConnectionRefused` errors to explicit diagnostics.
- The cross-customer fidelity test battery is a reusable regression gate for all
  future multi-tenant feature development.
- No Docker dependency: all isolation is achieved with loopback TCP, compatible
  with any CI environment that runs `cargo test`.
- Parallel startup via `tokio::join!` keeps harness build time well within the
  200ms budget even for 12-clone configurations (D-058).

### Negative

- New `prism-dtu-harness` crate adds one entry to the workspace. This is an
  intentional, scoped addition (housekeeping per ADR-012 accepts new crates that
  fulfill a clear new role; the harness fulfills the multi-tenant test
  infrastructure role that no existing crate covers).
- Logical mode still uses real TCP sockets (not fully in-memory). This is a deliberate
  choice: fully in-memory mocking would require a custom HTTP mock layer that
  diverges from the real `reqwest`-based sensor client path.
- `prism-dtu-harness` depends on all four client-mode DTU crates. If a DTU crate
  fails to compile, the harness also fails. This is correct behavior — the harness
  must compile against the actual clone implementations.

---

## 7. Behavioral Contracts Scoped by This ADR

| BC ID | Title | Postcondition summary |
|-------|-------|-----------------------|
| BC-3.5.001 | Harness logical isolation | In `IsolationMode::Logical`, a Prism MCP query scoped to `OrgId(A)` returns only records whose IDs carry the `OrgId(A)`-derived prefix. No records from any other org appear in the response. |
| BC-3.5.002 | Cross-customer non-leakage (network mode) | In `IsolationMode::Network`, the device ID sets across all registered orgs are pairwise disjoint. For all `OrgA ≠ OrgB`: `devices(OrgA) ∩ devices(OrgB) = ∅`. |
| BC-3.6.001 | Per-org failure injection | `Harness::inject_failure(org_slug, dtu_type, mode)` causes requests to `(org_slug, dtu_type)` to return the injected failure response, while requests to all other `(org, dtu)` pairs return normal responses. |
| BC-3.6.002 | Crash detection | If a clone task exits unexpectedly during a test, the next harness operation for that `(OrgId, DtuType)` returns `HarnessError::CloneCrashed` with the exit cause. No silent `ConnectionRefused` is propagated. |

---

## 8. Open Questions for Next Dispatch

1. **Port allocation retry strategy.** Three retries on `EADDRINUSE` is a reasonable
   default; is there a better strategy? One alternative: pre-allocate all ports at
   harness build time (before dropping any listeners) to minimize the race window.
   The `TcpListener` pre-allocation approach holds all listeners open simultaneously,
   then passes each to the corresponding clone — eliminating the race entirely at
   the cost of holding N listeners open during build. Recommend evaluating in
   implementation; pre-allocation is likely superior. **RESOLVED: pre-allocation selected via D-058; see §3.1 and Decision Refinements.**

2. **Shared-mode DTU in the harness.** ADR-006 Section 2.4 defines `mode = "shared"`
   for MSSP-internal DTUs (Slack, PagerDuty, Jira). The harness description in this ADR
   focuses on client-mode DTUs. Should shared-mode DTUs be represented in the harness,
   and if so, how? One shared clone per DTU type vs one per harness? Recommend:
   one shared clone per DTU type, accessible to all orgs in the harness, consistent
   with the `mode = "shared"` semantics. This is a Wave 3 story if Slack/PagerDuty/Jira
   DTU BC tests need multi-org harness support.

3. **`prism-dtu-harness` feature flag naming.** The crate is gated behind
   `#[cfg(any(test, feature = "dtu"))]` following `prism-dtu-common`'s convention.
   Should the feature flag be named `dtu` (matching existing convention) or `harness`
   (more specific)? Recommend `dtu` for consistency; spec-writer confirms in BC-3.5
   authoring.

4. **Harness reuse across test functions in `#[tokio::test]`.** If multiple test
   functions in one integration test file each build a `Harness`, port allocation
   happens N times. Is there a pattern for sharing a `Harness` across tests (e.g.,
   `once_cell` or Tokio `LazyLock`)? Recommend: no sharing in Wave 3. Each test
   builds its own harness for isolation. If harness startup time becomes a bottleneck,
   shared-harness patterns can be introduced in Wave 4.

---

## 9. ADR Chain — Related Documents

This ADR specifies the multi-tenant test harness that exercises the isolation
properties established in ADR-006 through ADR-008.

- **ADR-006:** Multi-tenant topology defines the `OrgId`/`OrgSlug` identity model
  and `client`/`shared` DTU mode. The harness `CustomerEndpoints` table is keyed
  by `(OrgId, DtuType)` as specified here.
- **ADR-007** (to be drafted): Per-DTU-type mode registry. Defines which DTU types
  are `client` vs `shared`, which determines which DTU types participate in the
  per-org harness topology.
- **ADR-008** (to be drafted): DTU state segregation. Defines the
  `HashMap<(OrgId, String), V>` keying pattern that the harness's cross-customer
  fidelity test verifies.
- **ADR-009:** Multi-tenant data generator. The harness calls `generate()` to
  populate each clone's initial fixture state before `start_on`.
- **ADR-012:** Workspace convention normalization. `prism-dtu-harness` follows
  the canonical `src/` crate layout defined there.

---

## Source / Origin

- **PO decision:** D-044 (network isolation NOT deferred) — recorded in
  `.factory/STATE.md`, Wave 3 kickoff 2026-04-27.
- **Code as-built — BehavioralClone trait:**
  `crates/prism-dtu-common/src/clone.rs` — `start_on(addr, shutdown, tls_config)`,
  `stop()`; the harness orchestrates multiple instances of this trait.
- **Code as-built — FailureLayer infrastructure:**
  `crates/prism-dtu-common/src/layers/failure.rs` — `FailureLayerShared`,
  `FailureMiddlewareShared`; per-clone dynamic failure injection used by
  `Harness::inject_failure`.
- **Code as-built — StubConfig:**
  `crates/prism-dtu-common/src/config.rs:5-31` — `StubConfig { seed, latency_ms,
  failure_mode, bind }`; `HarnessBuilder` produces one `StubConfig` per `(OrgId, DtuType)`.
- **Behavioral contracts:** BC-3.5.001, BC-3.5.002, BC-3.6.001, BC-3.6.002 — scoped
  by this ADR; to be authored by spec-writer in Phase 3.A.

---

## Decision Refinements (2026-04-27)

The following questions surfaced during BC authoring (Phase 3.A) and were resolved by the orchestrator on 2026-04-27. Each refinement is recorded here for historical traceability and is binding for Wave 3 implementation.

### D-058 — BC-3.5.001 parallel-startup latency budget tightened to 200ms

**Question:** Open Question 2 in Section 8 (v0.1) asked whether clone startup should be parallelized. BC-3.5.001 Postcondition 5 originally specified 500ms for a 3-org × 4-sensor (12-clone) harness build. What is the correct latency budget?

**Resolution:** The BC-3.5.001 parallel-startup latency budget tightens from 500ms to 200ms. Clone startup is parallelized via `tokio::join!` (or `futures::future::join_all`) — all `start_on` calls are issued concurrently, and the 200ms budget applies to the entire harness build completing. The original 500ms was a conservative estimate assuming sequential startup; parallel `tokio::join!` easily achieves 200ms for 12 clones on any CI runner.

**Rationale:** Sequential startup at 5s per clone timeout could theoretically block for 60s on a 12-clone harness. Parallel startup eliminates this concern. The `tokio::join!` pattern is idiomatic for Tokio-based concurrent initialization and has no correctness risk (each clone's `start_on` is independent). 200ms is achievable on all target CI environments (the existing single-clone `BehavioralClone::start_on` completes in under 10ms on loopback). This also closes Open Question 2 from the original ADR.

**Affected BCs:** BC-3.5.001 (Postcondition 5)

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 0.6 | 2026-04-27 | product-owner | M-003 (pass-6-remediation): Frontmatter `title:` corrected to Title Case to match H1 heading (POL 7 H1 source-of-truth). |
| 0.5 | 2026-04-27 | product-owner | M-005 (Pass 3): `subsystems_affected` updated [SS-05, SS-06] → [SS-01, SS-05, SS-06]. CAP-036/BC-3.5/3.6/3.7 BCs all anchor to SS-01 (Sensor Adapters); harness infrastructure necessarily touches the Sensor Adapter subsystem. |
| 0.4 | 2026-04-27 | product-owner | m-001 fix: added `anchored_capabilities: [CAP-036]` to frontmatter (per adversary Pass 2 minor finding). |
| 0.3 | 2026-04-27 | product-owner | C-1 sync: §3.1 threat model updated to reflect D-058 pre-allocation strategy; stale "retry up to 3 times" text replaced with pre-allocation-first mitigation that eliminates the race window. |
| 0.2 | 2026-04-27 | architect | Decision Refinements: D-058 (parallel-startup latency budget 500ms → 200ms via tokio::join!) |
| 0.1 | 2026-04-27 | architect | Initial draft — scopes D-044, D-045; status PROPOSED |
