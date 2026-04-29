---
document_type: holdout-scenario
level: L3
id: "HS-007"
category: "cross-repo-failure"
must_pass: true
priority: P1
epic_id: "E-3.6"
version: "2.0"
status: active
producer: implementer
timestamp: "2026-04-29T00:00:00Z"
phase: 3.A
inputs:
  - .factory/specs/behavioral-contracts/BC-3.5.001-harness-logical-isolation.md
  - .factory/specs/behavioral-contracts/BC-3.5.002-harness-network-isolation.md
  - .factory/specs/behavioral-contracts/BC-3.6.001-per-org-failure-injection.md
  - .factory/specs/behavioral-contracts/BC-3.6.002-harness-crash-detection.md
  - .factory/specs/architecture/decisions/ADR-011-harness-isolation-modes.md
input-hash: null
traces_to: prd.md
behavioral_contracts: [BC-3.5.001, BC-3.5.002, BC-3.6.001, BC-3.6.002]
closes_td: []
lifecycle_status: active
introduced: wave-3
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
notes: "Wave 3 re-anchor complete (S-3.6.02). Exercises per-org failure isolation in IsolationMode::Network harness: logical isolation invariants (BC-3.5.001), network-boundary routing verification (BC-3.5.002), per-org failure injection (BC-3.6.001), and crash detection (BC-3.6.002)."
---

# HS-007: Cross-Tenant Failure Isolation

**Group:** Per-customer DTU failure isolation; customer A fails, customer B unaffected
**Date:** 2026-04-29
**Priority:** P1
**Status:** Active — Wave 3 BC anchors installed (S-3.6.02)

## Scenario Overview

This holdout scenario exercises coordinated failure injection and detection in a
multi-tenant `prism-dtu-harness` environment running under `IsolationMode::Network`.
Two customer organizations (`acme-corp` and `globex`) each operate dedicated DTU clone
instances on distinct loopback ports. The scenario verifies three failure classes:

1. **Per-org failure isolation (HS-007-01):** Injecting `FailureMode::AuthReject` on
   `acme-corp`'s Claroty DTU clone causes HTTP 401 for that org only; `globex` continues
   receiving HTTP 200. Clearing the failure restores `acme-corp` to normal operation.
   Audit records are scoped to `acme-corp`'s `OrgId` — no audit event is written for
   `globex`.

2. **Network-boundary routing bug detection (HS-007-02):** A request bearing `acme-corp`
   credentials routed to `globex`'s dedicated `SocketAddr` returns HTTP 401 from
   `globex`'s authentication middleware. This makes cross-process routing bugs observable
   and distinguishable from legitimate failures.

3. **Crash detection with cross-tenant isolation (HS-007-03):** Injecting
   `FailureMode::InternalError` causes `acme-corp`'s Claroty clone to panic. The harness
   detects `HarnessError::CloneCrashed` for `acme-corp` within 1 second. `globex`'s clone
   continues returning HTTP 200 — no crash propagates across tenants.

All sub-scenarios require `IsolationMode::Network` (BC-3.5.002 precondition 1) with at
least two registered customer organizations (BC-3.5.002 precondition 2). The logical
isolation invariants from BC-3.5.001 — pairwise-disjoint device ID sets, independent
`FailureLayerShared` instances — apply within this network-mode harness.

---

## HS-007-01: Per-Org Failure Injection and Recovery

**Title:** Inject AuthReject on acme-corp Claroty DTU; globex remains healthy; clear restores acme-corp

### Preconditions

- A `prism-dtu-harness` is built with `IsolationMode::Network`
  (BC-3.5.002 precondition 1; BC-3.5.001 precondition 1 — logical isolation invariants
  apply in network mode).
- Two customer organizations are registered: `acme-corp` (Claroty clone) and `globex`
  (Claroty clone), each on a distinct OS-assigned loopback `SocketAddr`
  (BC-3.5.002 precondition 2).
- Both clones are running and healthy: `harness.customer_endpoints` contains entries for
  `(acme-corp, DtuType::Claroty)` and `(globex, DtuType::Claroty)` (BC-3.5.002
  precondition 3).
- Each clone's initial device IDs carry org-scoped prefixes per ADR-009:
  `dev-acme-corp-42-*` and `dev-globex-43-*` (BC-3.5.001 precondition 5).
- Each clone's `FailureLayerShared` instance is independent — no shared mutable state
  between clone instances (BC-3.6.001 precondition 3; BC-3.5.001 invariant 3).
- The `dtu` feature flag is enabled.

### Steps

1. Verify baseline: query `acme-corp`'s Claroty clone via its `SocketAddr`; assert HTTP 200
   and response body contains only device IDs with prefix `dev-acme-corp-42-`.
2. Verify baseline: query `globex`'s Claroty clone via its `SocketAddr`; assert HTTP 200
   and response body contains only device IDs with prefix `dev-globex-43-`.
3. Assert pairwise disjoint: `devices(acme-corp) ∩ devices(globex) = ∅`
   (BC-3.5.001 postcondition 2; BC-3.5.002 postcondition 1).
4. Call `harness.inject_failure("acme-corp", DtuType::Claroty, FailureMode::AuthReject)`;
   assert the call returns `Ok(())`.
5. Query `acme-corp`'s Claroty clone; assert HTTP 401
   (BC-3.6.001 postcondition 1 — `AuthReject` maps to HTTP 401 on every request).
6. Query `globex`'s Claroty clone; assert HTTP 200 with valid data
   (BC-3.6.001 postcondition 2 — other clones return normal responses;
   BC-3.5.001 invariant 3 — `acme-corp` failure injection does not alter `globex`'s
   `FailureLayerShared` state).
7. Assert that `globex`'s device IDs in the response still carry only `dev-globex-43-`
   prefixes — no cross-contamination from the failure-injected `acme-corp` clone.
8. Inspect `prism-audit` store: assert one audit event exists scoped to `acme-corp`'s
   `OrgId` recording `FailureMode::AuthReject` and a non-null timestamp. Assert zero
   audit events scoped to `globex`'s `OrgId` for this failure type
   (BC-3.6.001 postcondition 1 — per-org scoping of audit records).
9. Call `harness.clear_failure("acme-corp", DtuType::Claroty)`; assert `Ok(())`.
10. Query `acme-corp`'s Claroty clone; assert HTTP 200 with valid data
    (BC-3.6.001 postcondition 3 — `clear_failure` restores normal behavior).
11. Call `harness.clear_failure("acme-corp", DtuType::Claroty)` a second time; assert
    `Ok(())` — idempotent (BC-3.6.001 postcondition 4; BC-3.6.001 edge case EC-006).

### Expected Outcome

- `acme-corp`'s Claroty clone returns HTTP 401 while failure is injected; resumes HTTP 200
  after `clear_failure`.
- `globex`'s Claroty clone returns HTTP 200 throughout — never affected by `acme-corp`'s
  failure injection.
- Pairwise device ID disjointness holds before, during, and after failure injection.
- Audit record for `acme-corp` failure event present; no audit record for `globex`.
- `clear_failure` is idempotent: second call returns `Ok(())` without error.

### Failure Mode If Violated

- If `globex` receives HTTP 401 during `acme-corp` failure injection: `FailureLayerShared`
  state is shared across clones — BC-3.6.001 invariant 1 violated.
- If `acme-corp` returns HTTP 200 after `inject_failure`: failure injection not wired into
  the axum middleware stack — BC-3.6.001 postcondition 1 violated.
- If `clear_failure` returns an error on the second call: idempotency invariant broken —
  BC-3.6.001 postcondition 4 violated.
- If `globex` device IDs appear in `acme-corp`'s response (or vice versa): logical
  isolation breach — BC-3.5.001 postcondition 2 violated.

**BC Anchors:** [BC-3.5.001, BC-3.5.002, BC-3.6.001]

---

## HS-007-02: Network-Boundary Routing Bug Detection

**Title:** Wrong-org credentials routed to live clone endpoint return HTTP 401, exposing cross-process routing errors

### Preconditions

- A `prism-dtu-harness` is built with `IsolationMode::Network`
  (BC-3.5.002 precondition 1).
- Two customer organizations are registered: `acme-corp` (Claroty clone at `addr_A`) and
  `globex` (Claroty clone at `addr_B`), where `addr_A ≠ addr_B`
  (BC-3.5.002 postcondition 4 — all ports in `customer_endpoints` are pairwise distinct).
- Both clones are running and bound; `harness.customer_endpoints` is fully populated
  (BC-3.5.002 precondition 3).
- Each clone's authentication middleware is initialized with that clone's own `admin_token`;
  requests bearing a different org's token are rejected with HTTP 401
  (BC-3.5.002 precondition 6).
- No failure injection is active on either clone.

### Steps

1. Retrieve `addr_A` for `(acme-corp, DtuType::Claroty)` and `addr_B` for
   `(globex, DtuType::Claroty)` from `harness.customer_endpoints`.
2. Assert `addr_A ≠ addr_B` (BC-3.5.002 postcondition 4).
3. Send a well-formed HTTP GET request to `addr_A` bearing `acme-corp`'s credentials;
   assert HTTP 200 — correct routing, correct credentials.
4. Send a well-formed HTTP GET request to `addr_B` bearing `globex`'s credentials;
   assert HTTP 200 — correct routing, correct credentials.
5. **Routing bug simulation:** Send a well-formed HTTP GET request to `addr_B`
   (`globex`'s `SocketAddr`) bearing `acme-corp`'s credentials (wrong org).
6. Assert the response status is HTTP 401 — not HTTP 200, not a silent empty body, and
   not a `ConnectionRefused` error (BC-3.5.002 postcondition 2; BC-3.5.002 edge case
   EC-001 — cross-org credential mismatch returns HTTP 401).
7. Assert the HTTP 401 response body does not contain any `globex` device data — no data
   from the wrong org is leaked to the misdirected request.
8. Confirm `globex`'s own correctly-credentialed request still returns HTTP 200 after step
   6 — the routing probe did not destabilize the clone.

### Expected Outcome

- Correct-credential requests to each clone return HTTP 200.
- Wrong-org credentials sent to a live clone return HTTP 401.
- No `globex` data is returned to the `acme-corp`-credentialed request.
- `globex`'s clone remains healthy after the cross-org probe.
- The routing error is observable (HTTP 401 status) rather than silent (empty body or
  connection error), which makes cross-process routing bugs detectable in CI.

### Failure Mode If Violated

- If wrong-org credentials return HTTP 200 with data: the clone's authentication
  middleware is not validating org ownership — BC-3.5.002 postcondition 2 violated;
  multi-tenant data confidentiality breach.
- If wrong-org credentials return a silent empty body instead of HTTP 401: routing bugs
  become unobservable in CI — the purpose of `IsolationMode::Network` is defeated —
  BC-3.5.002 description violated.
- If the request results in `ConnectionRefused`: the clone was not running or its port was
  deallocated — BC-3.5.002 precondition 3 violated; harness startup incomplete.

**BC Anchors:** [BC-3.5.001, BC-3.5.002]

---

## HS-007-03: Crash Detection with Cross-Tenant Isolation

**Title:** InternalError injection causes acme-corp clone crash; harness detects CloneCrashed within 1s; globex unaffected

### Preconditions

- A `prism-dtu-harness` is built with `IsolationMode::Network`
  (BC-3.5.002 precondition 1).
- Two customer organizations are registered: `acme-corp` (Claroty clone) and `globex`
  (Claroty clone), each healthy at baseline (BC-3.5.002 precondition 3).
- Both clones' Tokio `JoinHandle`s are stored in the harness for crash monitoring
  (BC-3.6.002 precondition 1).
- Each clone's crash notification channel (`tokio::sync::watch` or equivalent) is wired
  and signals the harness before unwinding (BC-3.6.002 precondition 3).
- The `dtu` feature flag is enabled.

### Steps

1. Verify baseline: query `acme-corp`'s Claroty clone; assert HTTP 200.
2. Verify baseline: query `globex`'s Claroty clone; assert HTTP 200.
3. Call `harness.inject_failure("acme-corp", DtuType::Claroty, FailureMode::InternalError { after_n: 0 })`;
   this mode causes the clone's request handler to panic on the next request, triggering a
   Tokio task exit (BC-3.6.001 postcondition 1 — `InternalError` → HTTP 500 after N
   requests; when `after_n = 0`, the first request triggers the error path and the clone
   panics).
4. Send one HTTP GET request to `acme-corp`'s Claroty `SocketAddr` to trigger the
   `InternalError` path and induce the clone panic.
5. Wait up to 1 second for the harness to detect the crash via non-blocking `try_recv` on
   the crash notification channel (BC-3.6.002 postcondition 1 — detection within 1s of
   task exit).
6. Call any harness operation targeting `(acme-corp, DtuType::Claroty)` — for example,
   `harness.inject_failure("acme-corp", DtuType::Claroty, FailureMode::AuthReject)`.
7. Assert the call returns `Err(HarnessError::CloneCrashed { org_id, dtu_type, cause })`
   where `org_id = acme-corp`, `dtu_type = DtuType::Claroty`, and `cause` is a non-empty
   string (BC-3.6.002 postcondition 2 — `cause` contains panic message or error
   description; BC-3.6.002 postcondition 5 — error includes `OrgId`, `DtuType`, and
   diagnostic string).
8. Assert the `cause` string is non-empty (BC-3.6.002 invariant 4 — captured at exit
   time; if panic payload is not a string, `cause = "(non-string panic payload)"`).
9. Query `globex`'s Claroty clone; assert HTTP 200 with valid data
   (BC-3.6.002 postcondition 3 — clone crash does not affect other `(OrgId, DtuType)`
   pairs; BC-3.5.001 invariant 3 — failure injection on one clone does not alter others).
10. Assert `globex`'s device IDs in the response carry only `dev-globex-43-` prefixes —
    no cross-contamination from the crashed `acme-corp` clone.
11. Call `drop(harness)`; assert it completes without hanging or panicking
    (BC-3.6.002 postcondition 4 — drop after clone crash completes cleanly; no zombie
    Tokio tasks remain; BC-3.5.002 postcondition 6 — all TCP listeners released after drop).

### Expected Outcome

- `acme-corp`'s Claroty clone crashes after `FailureMode::InternalError` is triggered.
- The harness detects the crash within 1 second and marks `(acme-corp, DtuType::Claroty)`
  as crashed.
- Any subsequent harness operation targeting the crashed clone returns
  `Err(HarnessError::CloneCrashed)` with a non-empty `cause` string.
- `globex`'s Claroty clone continues returning HTTP 200 throughout — no crash propagates
  across tenant boundaries.
- `drop(harness)` completes cleanly — no leaked Tokio tasks, no zombie TCP listeners.

### Failure Mode If Violated

- If `globex` returns `CloneCrashed` after `acme-corp` crashes: crash state is not
  scoped per `(OrgId, DtuType)` — BC-3.6.002 postcondition 3 violated; cross-tenant crash
  propagation is a correctness failure.
- If the harness returns `ConnectionRefused` instead of `CloneCrashed`: crash detection is
  not implemented or the crash notification channel is not wired — BC-3.6.002 invariant 1
  violated (no operation may silently return `ConnectionRefused` for a crashed clone).
- If `CloneCrashed.cause` is empty or missing: diagnostic information is lost —
  BC-3.6.002 postcondition 5 violated; engineers cannot diagnose the root cause from test
  output.
- If `drop(harness)` hangs after the crash: zombie Tokio task or unjoined `JoinHandle` —
  BC-3.6.002 postcondition 4 violated.

**BC Anchors:** [BC-3.5.001, BC-3.5.002, BC-3.6.001, BC-3.6.002]

---

## State Checkpoint

```yaml
scenario_group: HS-007
title: Cross-Tenant Failure Isolation (Wave 3)
scenarios: 3
priority: P1
behavioral_contracts: [BC-3.5.001, BC-3.5.002, BC-3.6.001, BC-3.6.002]
closes_td: []
status: active
wave: 3
phase: 3.A
sub_scenarios:
  - id: HS-007-01
    title: "Per-Org Failure Injection and Recovery"
    bc_anchors: [BC-3.5.001, BC-3.5.002, BC-3.6.001]
  - id: HS-007-02
    title: "Network-Boundary Routing Bug Detection"
    bc_anchors: [BC-3.5.001, BC-3.5.002]
  - id: HS-007-03
    title: "Crash Detection with Cross-Tenant Isolation"
    bc_anchors: [BC-3.5.001, BC-3.5.002, BC-3.6.001, BC-3.6.002]
```
