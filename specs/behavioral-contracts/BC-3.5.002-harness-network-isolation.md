---
document_type: behavioral-contract
level: L3
bc_id: BC-3.5.002
title: Harness Network Isolation Invariants
version: "0.4"
status: PROPOSED
producer: product-owner
timestamp: 2026-04-27T00:00:00
phase: 3.A
wave: 3
inputs: [.factory/specs/architecture/decisions/ADR-011-harness-isolation-modes.md]
input-hash: "c1610fc"
traces_to: ".factory/specs/architecture/decisions/ADR-011-harness-isolation-modes.md"
origin: greenfield
extracted_from: null
subsystem: SS-01
capability: CAP-036
authors: [product-owner]
related_decisions: [D-044, D-045]
related_adrs: [ADR-011]
inherits_from: null
superseded_by: null
lifecycle_status: active
introduced: wave-3
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-3.5.002: Harness Network Isolation Invariants

## Description

In `IsolationMode::Network`, each `(OrgId, DtuType)` combination gets its own TCP listener
on an OS-assigned ephemeral port. The harness exposes a `customer_endpoints:
HashMap<(OrgId, DtuType), SocketAddr>` table; the Prism MCP sensor client uses this table
to route real HTTP requests over the loopback interface to each org's dedicated clone.
This mode catches cross-process routing bugs — specifically, a request bearing `OrgId(A)`
credentials that is accidentally routed to `OrgId(B)`'s port will receive an HTTP 401 from
`OrgId(B)`'s authentication middleware, making the routing error observable. Per D-044
(ADR-011 §1.1), network isolation ships in Wave 3 and is not deferred.

## Preconditions

1. `Harness::builder().isolation(IsolationMode::Network)` has been called.
2. At least two customer organizations are registered, each with at least one DTU type,
   to make cross-tenant routing verification meaningful.
3. `HarnessBuilder::build().await` has returned `Ok(harness)` — all clones are bound and
   their addresses are recorded in `harness.customer_endpoints`.
4. Port pre-allocation strategy is used: all `TcpListener` binds happen simultaneously
   before any `start_on` call, eliminating the bind-drop-rebind race window (ADR-011 §8 OQ-1).
5. The Prism MCP sensor client is configured with the full `customer_endpoints` table so
   that per-org requests are routed by `(OrgId, DtuType)` lookup — not by a shared
   in-process reference.
6. Each clone's authentication middleware is initialized with that clone's own
   `admin_token` (per ADR-003 Amendment §5); requests bearing a different org's token
   will be rejected with HTTP 401.
7. The `dtu` feature flag is enabled (crate gate: `#[cfg(any(test, feature = "dtu"))]`).

## Postconditions

1. For all pairs `(OrgA, OrgB)` where `OrgA ≠ OrgB`, the device ID sets returned by
   per-org queries are pairwise disjoint: `devices(OrgA) ∩ devices(OrgB) = ∅`.
2. A Prism MCP request bearing `OrgId(A)` credentials routed to `OrgId(B)`'s endpoint
   returns HTTP 401 — not a silent empty result or data from `OrgId(B)`.
3. A request to an `(OrgId, DtuType)` pair not present in `customer_endpoints` returns
   `HarnessError::UnknownOrg`; no request is forwarded to any live clone.
4. All ports in `customer_endpoints` are pairwise distinct (OS-assigned ephemeral ports;
   no two entries share a `SocketAddr`).
5. The entire harness — all clones across all orgs — completes `build().await` within 5
   seconds total (not per-clone); `tokio::join!` parallelizes startup per ADR-011 §2 and
   the locked decision in the Phase 3.A spec.
6. After `drop(harness)`, all TCP listeners bound during the harness lifetime are released;
   no clone task remains alive.

## Invariants

1. The `customer_endpoints` table is populated atomically during `build()` and is immutable
   for the harness lifetime.
2. Port allocation pre-allocates all listeners simultaneously before any `start_on` call;
   no retry-on-EADDRINUSE loop is used (locked decision, Phase 3.A spec).
3. Each clone's `FailureLayerShared` instance is independent; failure injection on
   `(OrgA, DtuType::X)` does not affect any other clone.
4. Network-mode clones use the same `BehavioralClone::start_on` path as logical-mode clones;
   the behavioral difference is exclusively in how the test client is configured to route
   requests (full `customer_endpoints` table vs shared in-process reference).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Cross-org credential mismatch (routing bug simulation) | Request with `OrgId(A)` credentials sent to `OrgId(B)` endpoint returns HTTP 401; no data from `OrgId(B)` is returned |
| EC-002 | Two parallel test runs with `--test-threads=4` | Each run receives independent OS-assigned port sets; no EADDRINUSE between runs |
| EC-003 | One clone fails to start within the 5s harness startup window | `build()` returns `Err(HarnessError::StartupTimeout)`; all other partially-started clones are aborted; no partial harness is returned |
| EC-004 | All ports for a 12-clone harness pre-allocated simultaneously | All 12 listeners bound before any `start_on`; zero EADDRINUSE race window; if OS cannot allocate 12 ephemeral ports, `build()` returns `Err(HarnessError::PortExhausted)` |
| EC-005 | Query to valid org but wrong DTU type (org has Claroty but query specifies Armis) | `customer_endpoints` lookup returns `None` for `(OrgA, DtuType::Armis)`; returns `HarnessError::UnknownDtuType` |
| EC-006 | Harness with a single org (degenerate case) | Postcondition 1 vacuously holds (no pairs to compare); other postconditions still apply |

## Canonical Test Vectors

| Scenario | Harness Config | Action | Expected Result | Pass Condition |
|----------|---------------|--------|----------------|----------------|
| TV-1: Pairwise disjoint IDs | acme-corp (Claroty), globex (Armis+CrowdStrike), initech (all 4) | Query all three orgs | Per-org device ID sets | `devices(acme) ∩ devices(globex) = ∅`, `devices(acme) ∩ devices(initech) = ∅`, `devices(globex) ∩ devices(initech) = ∅` |
| TV-2: Correct-endpoint routing | acme-corp (Claroty), globex (Claroty) | Query acme-corp via acme's endpoint | acme-corp device IDs only | No globex IDs in response; HTTP 200 |
| TV-3: Cross-org credential mismatch | acme-corp (Claroty), globex (Claroty) | Send acme-corp credentials to globex's SocketAddr | HTTP 401 | Response status 401; no data body |
| TV-4: Port uniqueness | 3-org × 4-sensor harness | Inspect `customer_endpoints` after `build()` | 12 distinct SocketAddrs | All 12 ports differ; no duplicates |
| TV-5: 5s total startup timeout | 3-org × 4-sensor harness | Time `build().await` | Completion time | Wall clock < 5s on CI runner |
| TV-6: Post-drop port release | acme-corp (Claroty) | Drop harness; probe port | `ConnectionRefused` | Within 1s of drop |
| TV-7: Unknown org | acme-corp registered only | Query globex | `HarnessError::UnknownOrg` | Error returned; no panic; no HTTP request sent |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-125 | All SocketAddrs in `customer_endpoints` are pairwise distinct after `build()` | proptest |
| VP-126 | A request with wrong-org credentials to a live clone returns HTTP 401, never HTTP 200 | integration test |
| VP-127 | `devices(OrgA) ∩ devices(OrgB) = ∅` for all registered org pairs in the 3-org canonical scenario | integration test (cross_tenant_isolation.rs per ADR-011 §2.9) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-036 ("Multi-Tenant DTU Test Harness") per capabilities.md §CAP-036 |
| Capability Anchor Justification | CAP-036 ("Multi-Tenant DTU Test Harness") per capabilities.md §CAP-036 — this BC specifies network-mode per-port clone orchestration and real-HTTP cross-tenant routing verification, the core purpose of the proposed CAP-036 harness capability. No existing CAP-001 through CAP-035 covers multi-tenant network-isolation test infrastructure. |
| L2 Domain Invariants | n/a (harness is test infrastructure; no DI-NNN enforced) |
| Architecture Module | prism-dtu-harness (ADR-011 §2.9); integration test: tests/cross_tenant_isolation.rs |
| Stories | S-3.3.04, S-3.3.05, S-3.4.01, S-3.4.02, S-3.4.03, S-3.4.04, S-3.6.02 |

## Related BCs

- BC-3.5.001 — logical-mode counterpart; provides fast unit-test coverage; this BC provides the network-boundary routing verification layer
- BC-3.6.001 — per-org failure injection is exercised within network-mode harness instances
- BC-3.6.002 — crash detection applies within network-mode harness instances

## Architecture Anchors

- `architecture/decisions/ADR-011-harness-isolation-modes.md#23-network-mode--per-port-os-process-isolation` — defines network-mode routing via `CustomerEndpoints` table
- `architecture/decisions/ADR-011-harness-isolation-modes.md#25-port-allocation-and-cleanup` — defines pre-allocation strategy (locked: simultaneous bind, no retry loop)
- `architecture/decisions/ADR-011-harness-isolation-modes.md#28-cross-customer-fidelity-test` — canonical 3-org scenario that implements TV-1 through TV-6

## Story Anchor

S-3.3.04, S-3.3.05, S-3.4.01, S-3.4.02, S-3.4.03, S-3.4.04, S-3.6.02

## VP Anchors

- VP-125 — proptest: all SocketAddrs in customer_endpoints are pairwise distinct after build()
- VP-126 — integration_test: wrong-org credentials to live clone returns HTTP 401, never HTTP 200
- VP-127 — integration_test: devices(OrgA) ∩ devices(OrgB) = ∅ for all org pairs in 3-org canonical scenario

## BC Changelog

| Version | Change |
|---------|--------|
| v0.4 | m-001 (Pass 6): `input-hash` populated: SHA1 of input file path (first 7 chars = `8606916`). |
| v0.3 | M-004/Audit-5 (Pass 5): Frontmatter `title:` corrected to title-case to match H1 heading. `traces_to:` corrected from `specs/domain-spec/capabilities.md` to `.factory/specs/architecture/decisions/ADR-011-harness-isolation-modes.md`. |
| v0.2 | Initial authoring from ADR-011. |
