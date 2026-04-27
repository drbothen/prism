---
document_type: behavioral-contract
level: L3
bc_id: BC-3.5.001
title: Harness logical isolation invariants
version: "0.2"
status: PROPOSED
producer: product-owner
timestamp: 2026-04-27T00:00:00
phase: 3.A
wave: 3
inputs: [.factory/specs/architecture/decisions/ADR-011-harness-isolation-modes.md]
input-hash: ""
traces_to: specs/domain-spec/capabilities.md
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

# BC-3.5.001: Harness Logical Isolation Invariants

## Description

In `IsolationMode::Logical`, the `prism-dtu-harness` crate spins all customer DTU clone
instances as Tokio tasks within a single OS process. Per-tenant state is segregated via
`(OrgId, DtuType)`-keyed HashMaps; each clone binds a distinct OS-assigned loopback TCP
port. This contract specifies that queries scoped to one organization return only that
organization's data, concurrent operations across tenants do not observe each other's
intermediate state, and harness teardown releases all in-process state cleanly.

## Preconditions

1. `Harness::builder().isolation(IsolationMode::Logical)` has been called.
2. At least one customer organization has been registered via `with_customer(slug, |c| ...)`.
3. `HarnessBuilder::build().await` has completed without error — all clone tasks are running
   and bound to their assigned loopback ports.
4. The Prism MCP sensor client used in the test is configured with the `Harness::endpoints`
   table so that it routes requests by `(OrgId, DtuType)` lookup.
5. Each clone's initial state was populated from the multi-tenant generator (ADR-009) with
   `OrgId`-tagged device IDs so that per-org prefixes are distinguishable in responses.
6. The `dtu` feature flag is enabled (crate gate: `#[cfg(any(test, feature = "dtu"))]`).

## Postconditions

1. A query scoped to `OrgId(A)` returns only records whose device IDs carry the `OrgId(A)`
   namespace prefix established at generator time. No records from any other organization
   appear in the response body.
2. For all pairs `(OrgA, OrgB)` where `OrgA ≠ OrgB`, the set intersection
   `devices(OrgA) ∩ devices(OrgB) = ∅` across all clone-returned device ID sets.
3. A concurrent read by `OrgA` during a write by `OrgB` returns a response consistent
   with `OrgA`'s own state at the time of the read; `OrgB`'s write does not appear in
   `OrgA`'s response.
4. After `drop(harness)` completes, no TCP listener remains bound on any port that was
   allocated to a clone instance; all Tokio tasks spawned by the harness have exited.
5. A 3-org × 4-sensor (12-clone) harness completes `build().await` in under 500ms on a
   standard CI runner (wall clock; conservative bound for sequential startup per ADR-011 §8 OQ-2).

## Invariants

1. Every `(OrgId, DtuType)` clone binds a distinct OS-assigned ephemeral port; no two
   clones in the same harness share a port.
2. The `endpoints` table is populated atomically during `build()` and is immutable for
   the harness lifetime — no entries are added or removed after `build()` returns.
3. Failure injection applied to `(OrgA, DtuType::X)` does not alter the `FailureLayerShared`
   state of any other `(OrgId, DtuType)` clone instance.
4. All harness-allocated ports are ephemeral (OS-assigned); no port is hardcoded or
   registered in a persistent port registry.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Caller queries an org slug not registered in the harness | `endpoints` lookup returns `None`; caller receives `HarnessError::UnknownOrg`; no HTTP request is sent |
| EC-002 | Single-org harness (one organization, one DTU type) | Behaves identically to multi-org case; `devices(OrgA)` is non-empty; teardown is clean |
| EC-003 | Clone fails to bind during `build()` — port unavailable | `build()` returns `Err(HarnessError::PortConflict { org, dtu })`; no partial `Harness` value is returned to the caller |
| EC-004 | `drop(harness)` while a clone is mid-request | Shutdown sender consumed; clone completes the in-flight request (graceful shutdown); exits within 5s; if not, `BehavioralClone::stop()` is called (hard abort) |
| EC-005 | All 12 clones started sequentially exceed 5s total startup | `build()` returns `Err(HarnessError::StartupTimeout)`; all partially-started tasks are aborted |
| EC-006 | Org A and Org B have zero overlapping device IDs (expected) | Assertion `devices(A) ∩ devices(B) = ∅` passes; no false positives from prefix collisions |

## Canonical Test Vectors

| Scenario | Harness Config | Query Org | Expected Device ID Prefix | Cross-Org IDs Present | Pass Condition |
|----------|---------------|-----------|--------------------------|----------------------|----------------|
| TV-1: Single-org baseline | acme-corp (Claroty, HealthyOtEnvironment) | acme-corp | `acme-corp/` | n/a | All returned IDs start with `acme-corp/`; count > 0 |
| TV-2: 3-org acme segregation | acme-corp (Claroty), globex (Armis+CrowdStrike), initech (all 4) | acme-corp | `acme-corp/` | none from globex or initech | Zero globex or initech IDs in acme-corp response |
| TV-3: 3-org globex segregation | same | globex | `globex/` | none from acme-corp or initech | Zero acme-corp or initech IDs |
| TV-4: 3-org initech segregation | same | initech | `initech/` | none from acme-corp or globex | Zero acme-corp or globex IDs |
| TV-5: Concurrent queries | acme-corp (Claroty), globex (Armis) | concurrent: both | per-org prefix | none cross-leaked | Pairwise-disjoint responses; no race-detected assertion failure |
| TV-6: Harness drop releases ports | acme-corp (Claroty) | n/a (post-drop probe) | n/a | n/a | `TcpStream::connect(clone_addr)` returns `ConnectionRefused` within 1s of drop |
| TV-7: Unknown org error | acme-corp (Claroty) | globex (not registered) | n/a | n/a | Returns `HarnessError::UnknownOrg`; no panic |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD-1 | `endpoints` map contains exactly `\|orgs\| × \|dtu_types_per_org\|` entries after `build()` | proptest |
| VP-TBD-2 | All socket addresses in `endpoints` are pairwise distinct (no port collision) | proptest |
| VP-TBD-3 | After `drop(harness)`, `TcpStream::connect` to every previously-recorded clone addr returns `ConnectionRefused` within 1s | integration test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-036 ("Multi-Tenant DTU Test Harness") per capabilities.md (proposed — no existing CAP covers multi-tenant harness infrastructure) |
| Capability Anchor Justification | CAP-036 ("Multi-Tenant DTU Test Harness") per capabilities.md §CAP-036 — this BC describes in-process per-org DTU clone orchestration and data segregation verification, which is precisely the scope of the proposed CAP-036 harness capability. No existing CAP (CAP-001 through CAP-035) covers multi-tenant test harness infrastructure. |
| L2 Domain Invariants | n/a (harness is test infrastructure; no DI-NNN enforced) |
| Architecture Module | prism-dtu-harness (ADR-011 §2.9) |
| Stories | TBD (filled by story-writer) |

## Related BCs

- BC-3.5.002 — network-mode counterpart; catches routing bugs that logical mode cannot
- BC-3.6.001 — depends on this BC's `(OrgId, DtuType)` per-clone structure for failure injection
- BC-3.6.002 — crash detection applies within logical-mode harness instances

## Architecture Anchors

- `architecture/decisions/ADR-011-harness-isolation-modes.md#22-logical-mode--in-process-org-keyed-routing` — defines `IsolationMode::Logical`, port allocation, and `Harness` struct layout
- `architecture/decisions/ADR-011-harness-isolation-modes.md#25-port-allocation-and-cleanup` — defines teardown behavior cited in Postcondition 4

## Story Anchor

TBD (filled by story-writer after story decomposition)

## VP Anchors

- VP-TBD-1, VP-TBD-2, VP-TBD-3 — to be assigned VP-NNN IDs by architect

## Open Questions

- Parallel startup latency budget (Postcondition 5): **Resolved — see ADR-011 §Decision Refinements (D-058).** Budget tightened from 500ms to 200ms; clone startup parallelized via `tokio::join!`. The 200ms budget applies to the entire 12-clone harness build.
