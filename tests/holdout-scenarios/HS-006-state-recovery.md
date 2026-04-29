---
document_type: holdout-scenario
level: L3
id: "HS-006"
category: "state-recovery"
must_pass: true
priority: P1
epic_id: "E-3.6"
version: "2.0"
status: active
producer: implementer
timestamp: "2026-04-29T00:00:00Z"
phase: "3.A"
inputs:
  - .factory/specs/behavioral-contracts/BC-3.6.001-per-org-failure-injection.md
  - .factory/specs/behavioral-contracts/BC-3.6.002-harness-crash-detection.md
  - .factory/specs/behavioral-contracts/BC-3.5.001-harness-logical-isolation.md
  - .factory/specs/behavioral-contracts/BC-3.2.001-per-org-sensor-data-isolation.md
  - .factory/specs/behavioral-contracts/BC-3.2.003-per-org-session-token-isolation.md
  - .factory/specs/architecture/decisions/ADR-011-harness-isolation-modes.md
behavioral_contracts: [BC-3.2.001, BC-3.2.003, BC-3.5.001, BC-3.6.001, BC-3.6.002]
closes_td: [TD-HOLDOUT-W2-002]
lifecycle_status: active
introduced: cycle-1
last_evaluated: "2026-04-29"
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
notes: "Wave 3 re-anchor: Phase 1b persistent-cursor language replaced with prism-dtu-harness RocksDB + crash detection model. All 7 sub-scenarios grounded in BC-3.6.001, BC-3.6.002, BC-3.5.001, BC-3.2.001, BC-3.2.003."
---

# HS-006: Multi-Tenant State Recovery Holdout Scenarios

**Group:** Multi-tenant harness restart without data loss; per-org state isolation preserved after DTU clone crash and recovery
**Date:** 2026-04-29
**Priority:** P1
**Status:** Active

## Overview

These scenarios validate that the `prism-dtu-harness` preserves per-organization state isolation
across DTU clone crashes, process restarts, and failure injection events. Each scenario involves
multiple tenant organizations (identified by `org_slug`) with state held in RocksDB
column families keyed by `(OrgId, resource_id)`. The harness manages per-org clone instances
as Tokio tasks whose `JoinHandle`s are monitored for unexpected exit.

The cross-tenant invariant asserted throughout is: `devices(OrgA) ∩ devices(OrgB) = ∅`
for all pairs `(OrgA, OrgB)` where `OrgA ≠ OrgB` (BC-3.5.001 postcondition 2).

---

## HS-006-01: Multi-Tenant Harness Restart with RocksDB State Persistence

**Title:** N-customer harness restarts cleanly; each org resumes from its last persisted RocksDB state without re-fetching already-delivered data.

**Preconditions:**
- A `Harness` with 3 organizations (`acme-corp`, `globex`, `initech`) and 2 DTU types each
  (Claroty, Armis) has completed an initial data collection cycle.
- Each org's device state is persisted to RocksDB column families keyed by `(OrgId, device_id)`.
- The harness process has been cleanly shut down (shutdown senders consumed; all Tokio tasks exited).

**Steps:**
1. Call `HarnessBuilder::build().await` to restart the harness with the same organization
   configuration and the same RocksDB data directory.
2. For each org, issue a sensor query via the `Harness::endpoints` table to verify the
   returned device set.
3. Record the device ID sets `devices(acme-corp)`, `devices(globex)`, `devices(initech)`.
4. Assert `devices(acme-corp) ∩ devices(globex) = ∅`.
5. Assert `devices(acme-corp) ∩ devices(initech) = ∅`.
6. Assert `devices(globex) ∩ devices(initech) = ∅`.
7. Verify that no org fetches data beyond the last persisted batch offset (query the
   RocksDB `last_offset` key for each org and confirm it matches the pre-restart value).

**Expected Outcome:**
- All 3 orgs return non-empty device sets whose IDs carry the org-specific namespace prefix
  established at generator time (BC-3.2.001 postcondition 1).
- The pairwise intersection of device ID sets across all org pairs is empty (BC-3.5.001 postcondition 2).
- No org-scoped RocksDB entry is present in another org's result set (BC-3.2.001 postcondition 2).
- The `last_offset` value for each org is unchanged from before the restart, confirming
  no duplicate data fetch occurred.

**Failure Mode If Violated:**
- Cross-org device IDs appear in a query response: structural isolation in the `(OrgId, device_id)`
  composite key is broken (BC-3.2.001 invariant 1 violated).
- Data is re-fetched from offset 0: RocksDB state was not persisted or was not loaded on restart.

**BC Anchors:** BC-3.2.001, BC-3.5.001

---

## HS-006-02: Clone Task Panic Mid-Operation; CloneCrashed Detection Within 1 Second

**Title:** When an org's DTU clone task panics mid-operation, the harness detects `CloneCrashed` within 1 second and other orgs' clones continue serving requests normally.

**Preconditions:**
- A `Harness` with 2 organizations (`acme-corp`, `globex`), each with a Claroty clone, is
  running via `IsolationMode::Logical`.
- Both clone Tokio tasks are healthy and returning HTTP 200 to sensor queries.
- The harness stores each clone's `JoinHandle` for crash monitoring (BC-3.6.002 precondition 2).

**Steps:**
1. Trigger a controlled panic in the `acme-corp` Claroty clone task using the harness test hook
   (e.g., send a panic-inducing message on the clone's internal shutdown channel with a
   non-graceful payload).
2. Wait up to 1 second for the harness crash-detection poll to observe the task exit.
3. Call any harness operation that targets `acme-corp` (e.g., a sensor query or `inject_failure`).
4. Issue a concurrent sensor query targeting `globex`.
5. Record the `cause` string from the returned `HarnessError::CloneCrashed`.

**Expected Outcome:**
- The harness operation targeting `acme-corp` returns `Err(HarnessError::CloneCrashed { org_id: "acme-corp", dtu_type: Claroty, cause })` where `cause` is a non-empty string containing the panic message (BC-3.6.002 postcondition 2).
- The crash is detected within 1 second of the clone task exiting (BC-3.6.002 postcondition 1).
- The sensor query targeting `globex` returns HTTP 200 with valid data; `globex`'s clone is unaffected (BC-3.6.002 postcondition 3).
- No device records from `acme-corp`'s last response appear in `globex`'s response: `devices(acme-corp) ∩ devices(globex) = ∅` (BC-3.5.001 postcondition 2).
- `drop(harness)` after the crash completes cleanly with no zombie Tokio tasks (BC-3.6.002 postcondition 4).

**Failure Mode If Violated:**
- The harness returns `Err(ConnectionRefused)` instead of `CloneCrashed`: crash detection is not
  wired (BC-3.6.002 invariant 1 violated).
- `globex` query returns data from `acme-corp`'s namespace: cross-tenant data leaked through
  in-memory clone state (BC-3.5.001 postcondition 2 violated).

**BC Anchors:** BC-3.6.002, BC-3.5.001

---

## HS-006-03: Query Fingerprint Mismatch Forces Per-Org State Reset

**Title:** When an org's sensor query configuration changes between harness runs, the RocksDB
state for that org is cleared and collection restarts from offset 0; other orgs' state is unaffected.

**Preconditions:**
- A `Harness` has completed one collection cycle for `acme-corp` (Claroty) and `globex` (Armis).
- RocksDB contains persisted state for both orgs, including per-org query fingerprints
  (`(OrgId, "query_fingerprint")` keys).
- The harness is stopped.

**Steps:**
1. Modify the Claroty sensor query configuration for `acme-corp` (e.g., change the device-filter
   field name), which alters the query fingerprint stored in RocksDB.
2. Restart the harness via `HarnessBuilder::build().await` with the updated configuration.
3. On startup, the harness computes the new query fingerprint for `acme-corp` and compares it
   against the value stored in RocksDB under `(OrgId("acme-corp"), "query_fingerprint")`.
4. Because the fingerprints differ, the harness clears `acme-corp`'s RocksDB state and sets
   the collection offset to 0.
5. Issue a sensor query to `globex` immediately after restart.

**Expected Outcome:**
- `acme-corp` begins re-fetching device data from offset 0; its RocksDB state is empty at
  startup and is repopulated during the first collection cycle (no stale data served).
- `globex`'s RocksDB state is not modified by `acme-corp`'s fingerprint-mismatch reset;
  `globex` resumes from its persisted offset (BC-3.2.001 postcondition 2).
- `devices(acme-corp) ∩ devices(globex) = ∅` after both orgs have completed their first
  post-restart collection cycle (BC-3.5.001 postcondition 2).
- No device ID from `globex`'s RocksDB namespace appears in `acme-corp`'s query response
  during the re-fetch period (BC-3.2.001 postcondition 1).

**Failure Mode If Violated:**
- `globex` state is cleared alongside `acme-corp`: the fingerprint reset is not scoped to
  the affected org (BC-3.2.001 postcondition 2 violated).
- Stale pre-fingerprint-change data is served for `acme-corp`: the state reset did not occur.

**BC Anchors:** BC-3.2.001, BC-3.5.001

---

## HS-006-04: Per-Org RocksDB Offset Is Monotonically Non-Decreasing

**Title:** The harness rejects any attempt to set an org's collection offset to a value
lower than its current persisted offset, preventing duplicate data delivery.

**Preconditions:**
- A `Harness` with `acme-corp` (CrowdStrike) is running.
- `acme-corp`'s RocksDB column family contains a persisted offset of `N` (e.g., N=1000
  events delivered).
- The harness exposes a `set_offset(org_id, offset)` internal API used during recovery
  and fingerprint-mismatch resets.

**Steps:**
1. Call `set_offset(OrgId("acme-corp"), 500)` — a value less than the current persisted offset of 1000.
2. Verify the return value.
3. Issue a sensor query to `acme-corp` and record the first event ID returned.
4. Call `set_offset(OrgId("acme-corp"), 2000)` — a value greater than the current offset.
5. Issue another sensor query and record the first event ID returned.

**Expected Outcome:**
- `set_offset(acme-corp, 500)` returns `Err(HarnessError::OffsetRegression { org_id: "acme-corp", current: 1000, attempted: 500 })`;
  the persisted offset remains 1000 (forward progress invariant enforced).
- The sensor query after the rejected regression still serves events from offset 1000 onward,
  not from 500 (no data re-delivery).
- `set_offset(acme-corp, 2000)` returns `Ok(())`; the subsequent query begins from offset 2000
  (monotonic advance permitted).
- No other org's offset is modified by either call (BC-3.2.001 postcondition 2).

**Failure Mode If Violated:**
- Backward offset accepted silently: events already delivered are re-delivered, violating
  the forward progress invariant.
- Another org's offset is modified: composite key isolation broken (BC-3.2.001 invariant 1 violated).

**BC Anchors:** BC-3.2.001, BC-3.2.003

---

## HS-006-05: Per-Org Session Token Survives Harness Restart; Cross-Org Tokens Remain Isolated

**Title:** After a harness restart, each org's session/bearer tokens are re-loaded from the
per-org token store and remain isolated: a token registered under OrgA is not accepted for OrgB.

**Preconditions:**
- A `Harness` with 2 orgs (`acme-corp` with a CrowdStrike OAuth token, `globex` with a
  Cyberint session UUID) has run one collection cycle.
- Both tokens are persisted in the RocksDB session store keyed by `(OrgId, token_string)`
  (BC-3.2.003 precondition 1).
- The harness is cleanly stopped.

**Steps:**
1. Restart the harness via `HarnessBuilder::build().await`.
2. Call `is_valid_session(OrgId("acme-corp"), acme_token)` using the token previously
   registered for `acme-corp`.
3. Call `is_valid_session(OrgId("globex"), acme_token)` using the same token string but
   in `globex`'s context.
4. Call `is_valid_session(OrgId("globex"), globex_token)` using `globex`'s own token.
5. Simulate token expiry for `acme-corp` and trigger a refresh: verify the new token is
   stored under `OrgId("acme-corp")` and `globex`'s token is unchanged.

**Expected Outcome:**
- `is_valid_session(acme-corp, acme_token)` returns valid after restart (token persisted correctly).
- `is_valid_session(globex, acme_token)` returns invalid: a token registered under `acme-corp`
  is not accepted for `globex`, even if the string is identical (BC-3.2.003 postcondition 2).
- `is_valid_session(globex, globex_token)` returns valid.
- After `acme-corp` token refresh, the new token is stored under `OrgId("acme-corp")`; `globex_token`
  is unchanged and still valid (BC-3.2.003 postcondition 4).
- `devices(acme-corp) ∩ devices(globex) = ∅` across concurrent sensor queries during token
  refresh (BC-3.5.001 postcondition 2).

**Failure Mode If Violated:**
- `acme_token` accepted for `globex`: token store not keyed by `(OrgId, token_string)`
  (BC-3.2.003 invariant 1 violated).
- `globex_token` invalidated by `acme-corp` token refresh: token refresh flow does not preserve
  OrgId binding (BC-3.2.003 postcondition 3 violated).

**BC Anchors:** BC-3.2.003, BC-3.5.001

---

## HS-006-06: Simultaneous Multi-Org Clone Crash; Independent Recovery; Cross-Tenant Integrity Preserved

**Title:** When multiple org clones crash simultaneously, each is detected and reported independently,
non-crashed clones continue serving requests, and no cross-tenant data contamination occurs.

**Preconditions:**
- A `Harness` with 4 organizations (`acme-corp`, `globex`, `initech`, `umbrella`) each with
  a Claroty clone and an Armis clone (8 clones total) is running via `IsolationMode::Logical`.
- All 8 clone Tokio tasks are healthy and their `JoinHandle`s are stored in the harness.
- RocksDB contains persisted state for all 4 orgs.

**Steps:**
1. Simultaneously trigger panics in the `acme-corp` Claroty clone and the `globex` Armis clone
   using the harness test hooks (two concurrent panic inductions).
2. Wait up to 1 second for crash detection to mark both crashed clones.
3. Issue sensor queries targeting `acme-corp` (Claroty) and `globex` (Armis) in sequence.
4. Issue sensor queries targeting `initech` (Claroty) and `umbrella` (Armis) in sequence.
5. Record the device sets returned by `initech` and `umbrella` queries.
6. Assert cross-tenant isolation across all 4 orgs.

**Expected Outcome:**
- Queries to `acme-corp` (Claroty) and `globex` (Armis) return `Err(HarnessError::CloneCrashed { ... })`
  for each respective crashed clone (BC-3.6.002 postcondition 2).
- Both crashes are detected independently within 1 second of each task exiting; each `CloneCrashed`
  carries the correct `org_id` and `dtu_type` (BC-3.6.002 postcondition 1 and postcondition 5).
- Queries to `initech` and `umbrella` return HTTP 200 with valid data; neither is affected by
  the two simultaneous crashes (BC-3.6.002 postcondition 3).
- `devices(initech) ∩ devices(umbrella) = ∅`; no device ID from any crashed org appears in
  `initech` or `umbrella` responses (BC-3.5.001 postcondition 2).
- `drop(harness)` completes cleanly; no zombie Tokio tasks from the crashed clones remain
  (BC-3.6.002 postcondition 4).

**Failure Mode If Violated:**
- Only one crash detected when two occurred simultaneously: crash detection is not concurrent
  (BC-3.6.002 edge case EC-005 violated).
- `initech` or `umbrella` query returns data from a crashed org's namespace: in-memory
  state contamination through shared clone resources (BC-3.5.001 invariant 3 violated).

**BC Anchors:** BC-3.6.002, BC-3.5.001, BC-3.2.001

---

## HS-006-07: Per-Org Failure Injection Triggers Crash Detection; Sibling Org Unaffected

**Title:** Injecting `FailureMode::InternalError` on OrgA's clone causes that clone to crash;
the harness detects `CloneCrashed` for OrgA only; OrgB's clone continues returning HTTP 200
and its state remains isolated.

**Preconditions:**
- A `Harness` with 2 organizations (`acme-corp`, `globex`), each with a Cyberint clone,
  is running via `IsolationMode::Logical`.
- Both clones are healthy; their `FailureLayerShared` instances are initialized and wired
  into the axum middleware stack (BC-3.6.001 precondition 3).
- The `dtu` feature flag is enabled.

**Steps:**
1. Call `harness.inject_failure("acme-corp", DtuType::Cyberint, FailureMode::InternalError { after_n: 0 })`.
2. Verify `inject_failure` returns `Ok(())`.
3. Issue a sensor query to `acme-corp` (Cyberint); the injected HTTP 500 causes the clone
   task to exit with an error.
4. Wait up to 1 second for the harness to detect the clone task exit.
5. Issue a second sensor query to `acme-corp` (Cyberint).
6. Issue a concurrent sensor query to `globex` (Cyberint).
7. Assert the cross-tenant isolation invariant.

**Expected Outcome:**
- `inject_failure("acme-corp", Cyberint, InternalError { after_n: 0 })` returns `Ok(())`;
  only `acme-corp`'s `FailureLayerShared` state is modified; `globex`'s `FailureLayerShared`
  is unchanged (BC-3.6.001 postcondition 2).
- The second query to `acme-corp` returns `Err(HarnessError::CloneCrashed { org_id: "acme-corp", dtu_type: Cyberint, cause })` where `cause` is non-empty (BC-3.6.002 postcondition 2).
- The crash is detected within 1 second of the clone task exiting (BC-3.6.002 postcondition 1).
- The concurrent query to `globex` returns HTTP 200 with valid Cyberint data; `globex`'s clone
  is entirely unaffected by `acme-corp`'s failure injection and crash (BC-3.6.001 postcondition 2
  and BC-3.6.002 postcondition 3).
- `devices(acme-corp) ∩ devices(globex) = ∅`; no data from `acme-corp`'s Cyberint namespace
  appears in `globex`'s response (BC-3.5.001 postcondition 2).
- Calling `inject_failure("acme-corp", Cyberint, AuthReject)` after the crash returns
  `Err(HarnessError::CloneCrashed { org_id: "acme-corp", ... })` — the harness refuses to
  communicate with a dead clone rather than silently making an HTTP call (BC-3.6.001 edge case EC-004).

**Failure Mode If Violated:**
- `inject_failure` on `acme-corp` also modifies `globex`'s `FailureLayerShared`: shared mutable
  state exists between clone instances (BC-3.6.001 invariant 1 violated).
- `globex` query returns `CloneCrashed` after `acme-corp` crash: crash state is shared across
  clones (BC-3.6.002 postcondition 3 violated).
- The harness returns `ConnectionRefused` instead of `CloneCrashed` for `acme-corp`: crash
  detection not wired (BC-3.6.002 invariant 1 violated).

**BC Anchors:** BC-3.6.001, BC-3.6.002, BC-3.5.001

---

## State Checkpoint

```yaml
scenario_group: HS-006
title: Multi-Tenant State Recovery (Wave 3)
scenarios: 7
priority: P1
behavioral_contracts: [BC-3.2.001, BC-3.2.003, BC-3.5.001, BC-3.6.001, BC-3.6.002]
closes_td: [TD-HOLDOUT-W2-002]
status: active
wave: 3
phase: 3.A
```
